use crate::{
    args::Args,
    test::{Outcome, OutcomeKind, Test, TestDesc, TestKind},
};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use futures_util::{ready, stream::StreamExt};
use pin_project_lite::pin_project;
use std::{collections::HashSet, pin::Pin};

pin_project! {
    struct PendingTest<D, R> {
        desc: TestDesc,
        context: Option<D>,
        #[pin]
        test_case: Option<R>,
        outcome: Option<Outcome>,
        name_length: usize,
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
                match outcome.kind() {
                    OutcomeKind::Passed => println!(
                        "test {0:<1$} ... {2}",
                        me.desc.name(),
                        me.name_length,
                        console::style("ok").green()
                    ),
                    OutcomeKind::Failed => println!(
                        "test {0:<1$} ... {2}",
                        me.desc.name(),
                        me.name_length,
                        console::style("FAILED").red()
                    ),
                    OutcomeKind::Measured { average, variance } => println!(
                        "test {0:<1$} ... {2}: {3} nsec/iter (+/- {4})",
                        me.desc.name(),
                        me.name_length,
                        console::style("bench").cyan(),
                        average,
                        variance
                    ),
                }
                me.outcome.replace(outcome);
            }
            None => {
                println!(
                    "test {0:<1$} ... {2}",
                    me.desc.name(),
                    me.name_length,
                    console::style("ignored").yellow()
                );
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

    fn print_list<D, R>(&self, tests: &[PendingTest<D, R>]) {
        let quiet = self.args.format == crate::args::OutputFormat::Terse;

        let mut num_tests = 0;
        let mut num_benches = 0;

        for test in tests {
            let kind_str = match test.desc.kind() {
                TestKind::Test => {
                    num_tests += 1;
                    "test"
                }
                TestKind::Bench => {
                    num_benches += 1;
                    "benchmark"
                }
            };
            println!("{}: {}", test.desc.name(), kind_str);
        }

        if !quiet {
            fn plural_suffix(n: usize) -> &'static str {
                match n {
                    1 => "",
                    _ => "s",
                }
            }

            if num_tests != 0 || num_benches != 0 {
                println!();
            }
            println!(
                "{} test{}, {} benchmark{}",
                num_tests,
                plural_suffix(num_tests),
                num_benches,
                plural_suffix(num_benches)
            );
        }
    }

    fn apply_filter<D, R>(
        &self,
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
                name_length: 0,
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

        let (mut pending_tests, num_filtered_out) = self.apply_filter(tests);

        if self.args.list {
            self.print_list(&pending_tests);
            return 0;
        }

        println!("running {} tests", pending_tests.len());

        let max_name_length = pending_tests
            .iter()
            .map(|test| test.desc.name().len())
            .max()
            .unwrap_or(0);

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
            test.name_length = max_name_length;
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

        let mut status = console::style("ok").green();
        if !failed_tests.is_empty() {
            status = console::style("FAILED").red();
            println!();
            println!("failures:\n");
            for (name, msg) in &failed_tests {
                println!("---- {} ----", name);
                if let Some(msg) = msg {
                    println!("{}\n", msg);
                }
            }
        }

        println!();
        println!("test result: {status}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out",
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
