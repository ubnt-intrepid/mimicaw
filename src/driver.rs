use crate::{
    args::Args,
    printer::Printer,
    test::{Outcome, OutcomeKind, Test, TestDesc, TestKind},
};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use futures_util::{ready, stream::StreamExt};
use pin_project_lite::pin_project;
use std::{collections::HashSet, io::Write, pin::Pin, sync::Arc};

pin_project! {
    struct PendingTest<D, R> {
        desc: TestDesc,
        context: Option<D>,
        #[pin]
        test_case: Option<R>,
        outcome: Option<Outcome>,
        printer: Arc<Printer>,
    }
}

impl<D, R> Future for PendingTest<D, R>
where
    R: Future<Output = Outcome>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();

        match me.test_case.as_pin_mut() {
            Some(test_case) => {
                let outcome = ready!(test_case.poll(cx));
                me.printer.print_result(me.desc, Some(&outcome));
                me.outcome.replace(outcome);
            }
            None => {
                me.printer.print_result(me.desc, None);
            }
        }

        Poll::Ready(())
    }
}

/// The test driver.
pub struct TestDriver {
    args: Args,
}

impl TestDriver {
    /// Create a test driver configured with the CLI options.
    pub fn from_env() -> Result<Self, i32> {
        Ok(Self {
            args: Args::from_env()?,
        })
    }

    fn apply_filter<D, R>(
        &self,
        printer: &Arc<Printer>,
        tests: impl IntoIterator<Item = Test<D>>,
    ) -> (Vec<PendingTest<D, R>>, usize) {
        let mut pending_tests = vec![];
        let mut num_filtered_out = 0;
        let mut test_names = HashSet::new();

        for test in tests {
            let (desc, context) = test.deconstruct();
            if !test_names.insert(desc.name_arc().clone()) {
                panic!("the test name is conflicted");
            }

            if self.args.is_filtered(desc.name()) {
                num_filtered_out += 1;
                continue;
            }

            pending_tests.push(PendingTest {
                desc,
                context: Some(context),
                test_case: None,
                outcome: None,
                printer: printer.clone(),
            });
        }

        (pending_tests, num_filtered_out)
    }

    /// Run a set of tests using the specified test runner.
    pub async fn run_tests<D, I, F, R>(&mut self, tests: I, runner: F) -> i32
    where
        I: IntoIterator<Item = Test<D>>,
        F: FnMut(&TestDesc, D) -> R,
        R: Future<Output = Outcome> + Unpin,
    {
        let mut runner = runner;

        let printer = Arc::new(Printer::new(&self.args));

        let (mut pending_tests, num_filtered_out) = self.apply_filter(&printer, tests);

        if self.args.list {
            printer.print_list(pending_tests.iter().map(|test| &test.desc));
            return 0;
        }

        let _ = writeln!(&mut printer.term(), "running {} tests", pending_tests.len());

        let max_name_length = pending_tests
            .iter()
            .map(|test| test.desc.name().len())
            .max()
            .unwrap_or(0);
        printer.set_name_length(max_name_length);

        for test in &mut pending_tests {
            let ignored = (test.desc.ignored() && !self.args.run_ignored)
                || match test.desc.kind() {
                    TestKind::Test => !self.args.run_tests,
                    TestKind::Bench => !self.args.run_benchmarks,
                };

            let context = test
                .context
                .take()
                .expect("the context has already been used");
            if !ignored {
                test.test_case.replace(runner(&test.desc, context));
            }
        }

        futures_util::stream::iter(pending_tests.iter_mut()) //
            .for_each_concurrent(None, std::convert::identity)
            .await;

        let mut num_passed = 0;
        let mut failed_tests = vec![];
        let mut num_measured = 0;
        let mut num_ignored = 0;
        for test in pending_tests {
            match test.outcome {
                Some(outcome) => match outcome.kind() {
                    OutcomeKind::Passed => num_passed += 1,
                    OutcomeKind::Failed => {
                        failed_tests.push((test.desc.name_arc().clone(), outcome.err_msg()))
                    }
                    OutcomeKind::Measured { .. } => num_measured += 1,
                },
                None => num_ignored += 1,
            }
        }

        let mut status = printer.styled("ok").green();
        if !failed_tests.is_empty() {
            status = printer.styled("FAILED").red();
            let _ = writeln!(&mut printer.term());
            let _ = writeln!(&mut printer.term(), "failures:\n");
            for (name, msg) in &failed_tests {
                let _ = writeln!(&mut printer.term(), "---- {} ----", name);
                if let Some(msg) = msg {
                    let _ = writeln!(&mut printer.term(), "{}\n", msg);
                }
            }
        }

        let _ = writeln!(&mut printer.term());
        let _ = writeln!(&mut printer.term(), "test result: {status}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out",
            status = status,
            passed = num_passed,
            failed = failed_tests.len(),
            ignored = num_ignored,
            measured = num_measured,
            filtered_out = num_filtered_out,
        );

        if failed_tests.is_empty() {
            0
        } else {
            crate::ERROR_STATUS_CODE
        }
    }
}
