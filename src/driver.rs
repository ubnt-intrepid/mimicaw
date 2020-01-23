use crate::{
    args::Args,
    printer::Printer,
    test::{Outcome, OutcomeKind, Test, TestDesc, TestKind},
    ExitStatus,
};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use futures_util::{ready, stream::StreamExt};
use pin_project_lite::pin_project;
use std::{collections::HashSet, io::Write, pin::Pin};

/// The runner of test cases.
pub trait TestRunner<D> {
    /// The type of future returned from `run`.
    type Future: Future<Output = Outcome> + Unpin;

    /// Run a test case.
    fn run(&mut self, desc: TestDesc, data: D) -> Self::Future;
}

impl<F, D, R> TestRunner<D> for F
where
    F: FnMut(TestDesc, D) -> R,
    R: Future<Output = Outcome> + Unpin,
{
    type Future = R;

    fn run(&mut self, desc: TestDesc, data: D) -> Self::Future {
        (*self)(desc, data)
    }
}

pin_project! {
    struct PendingTest<'a, D, R> {
        desc: TestDesc,
        context: Option<D>,
        #[pin]
        test_case: Option<R>,
        outcome: Option<Outcome>,
        printer: &'a Printer,
        name_length: usize,
    }
}

impl<D, R> Future for PendingTest<'_, D, R>
where
    R: Future<Output = Outcome>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();

        match me.test_case.as_pin_mut() {
            Some(test_case) => {
                let outcome = ready!(test_case.poll(cx));
                me.printer
                    .print_result(me.desc, *me.name_length, Some(&outcome));
                me.outcome.replace(outcome);
            }
            None => {
                me.printer.print_result(me.desc, *me.name_length, None);
            }
        }

        Poll::Ready(())
    }
}

pub(crate) struct TestDriver<'a> {
    args: &'a Args,
    printer: Printer,
}

impl<'a> TestDriver<'a> {
    pub(crate) fn new(args: &'a Args) -> Self {
        let printer = Printer::new(&args);
        Self { args, printer }
    }

    pub(crate) async fn run_tests<D>(
        &self,
        tests: impl IntoIterator<Item = Test<D>>,
        runner: impl TestRunner<D>,
    ) -> ExitStatus {
        let mut runner = runner;

        let (mut pending_tests, num_filtered_out) = {
            let mut pending_tests = vec![];
            let mut num_filtered_out = 0_usize;
            let mut test_names = HashSet::new();

            for test in tests {
                let (desc, context) = test.deconstruct();
                if !test_names.insert(desc.name().to_string()) {
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
                    printer: &self.printer,
                    name_length: 0,
                });
            }

            (pending_tests, num_filtered_out)
        };

        if self.args.list {
            self.printer
                .print_list(pending_tests.iter().map(|test| &test.desc));
            return ExitStatus::OK;
        }

        let _ = writeln!(self.printer.term(), "running {} tests", pending_tests.len());

        let max_name_length = pending_tests
            .iter()
            .map(|test| test.desc.name().len())
            .max()
            .unwrap_or(0);

        for test in &mut pending_tests {
            test.name_length = max_name_length;

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
                test.test_case
                    .replace(runner.run(test.desc.clone(), context));
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
                        failed_tests.push((test.desc.clone(), outcome.err_msg()))
                    }
                    OutcomeKind::Measured { .. } => num_measured += 1,
                },
                None => num_ignored += 1,
            }
        }

        let mut status = self.printer.styled("ok").green();
        if !failed_tests.is_empty() {
            status = self.printer.styled("FAILED").red();
            let _ = writeln!(self.printer.term());
            let _ = writeln!(self.printer.term(), "failures:\n");
            for (desc, msg) in &failed_tests {
                let _ = writeln!(self.printer.term(), "---- {} ----", desc.name());
                if let Some(msg) = msg {
                    let _ = writeln!(self.printer.term(), "{}\n", msg);
                }
            }
        }

        let _ = writeln!(self.printer.term());
        let _ = writeln!(self.printer.term(), "test result: {status}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out",
            status = status,
            passed = num_passed,
            failed = failed_tests.len(),
            ignored = num_ignored,
            measured = num_measured,
            filtered_out = num_filtered_out,
        );

        if failed_tests.is_empty() {
            ExitStatus::OK
        } else {
            ExitStatus::FAILED
        }
    }
}
