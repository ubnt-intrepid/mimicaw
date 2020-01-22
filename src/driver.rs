use crate::{
    args::Args,
    progress::{Container, Progress},
    test::{Outcome, Test, TestKind},
};
use futures::{
    future::Future,
    stream::StreamExt,
    task::{self, Poll},
};
use pin_project_lite::pin_project;
use std::{collections::HashSet, pin::Pin};

pin_project! {
    struct RunningTest<D, R> {
        test: Test<D>,
        filtered: bool,
        #[pin]
        test_case: R,
        progress: Progress,
        outcome: Option<Outcome>,
    }
}

impl<D, R> Future for RunningTest<D, R>
where
    R: Future<Output = Outcome>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        if *me.filtered {
            me.progress.finish(None);
            return Poll::Ready(());
        }

        me.progress.set_running();

        let outcome = futures::ready!(me.test_case.poll(cx));
        me.progress.finish(Some(&outcome));
        me.outcome.replace(outcome);

        Poll::Ready(())
    }
}

pub struct TestDriver {
    args: Args,
}

impl TestDriver {
    /// Create a test suite.
    pub fn from_env() -> Self {
        match Args::from_env() {
            Ok(args) => Self { args },
            Err(code) => {
                // The process should not be exited at here
                // in order for the resources in main function to
                // be appropriately dropped.
                std::process::exit(code);
            }
        }
    }

    fn print_list<D>(&self, tests: impl IntoIterator<Item = Test<D>>) {
        let quiet = self.args.format == crate::args::OutputFormat::Terse;

        let mut num_tests = 0;
        let mut num_benches = 0;

        for test in tests {
            let kind_str = match test.kind() {
                TestKind::Test => {
                    num_tests += 1;
                    "test"
                }
                TestKind::Bench => {
                    num_benches += 1;
                    "benchmark"
                }
            };
            println!("{}: {}", test.name(), kind_str);
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

    /// Run the test suite and aggregate the results.
    pub async fn run_tests<D, I, F, R>(&mut self, tests: I, runner: F) -> i32
    where
        I: IntoIterator<Item = Test<D>>,
        F: FnMut(D) -> R,
        R: Future<Output = Outcome> + Unpin,
    {
        let mut runner = runner;

        if self.args.list {
            self.print_list(tests);
            return 0;
        }

        let mut running_tests = vec![];
        let mut test_names = HashSet::new();
        let mut max_name_length = 0;

        for test in tests {
            if !test_names.insert(test.name().clone()) {
                panic!("the test name is conflicted");
            }

            max_name_length = std::cmp::max(max_name_length, test.name().len());

            let is_target_mode = match test.kind() {
                TestKind::Test => self.args.run_tests,
                TestKind::Bench => self.args.run_benchmarks,
            };
            let filtered =
                test.ignored() || !is_target_mode || self.args.is_filtered(&*test.name());
            let filtered = filtered ^ self.args.run_ignored;

            running_tests.push((test, filtered));
        }

        println!("running {} tests", running_tests.len());
        let container = Container::new(max_name_length);

        let mut running_tests: Vec<_> = running_tests
            .into_iter()
            .map(|(mut test, filtered)| {
                let test_case = runner(test.take_context());
                let progress = container.add_progress(&*test.name());
                RunningTest {
                    test,
                    filtered,
                    test_case,
                    progress,
                    outcome: None,
                }
            })
            .collect();

        let run_tests = futures::stream::iter(running_tests.iter_mut()) //
            .for_each_concurrent(1024, std::convert::identity);
        let complete_progress = container.join();
        let _ = futures::future::join(run_tests, complete_progress).await;

        let mut passed_tests = vec![];
        let mut failed_tests = vec![];
        let mut benchmark_tests = vec![];
        let mut ignored_tests = vec![];
        for test in running_tests {
            match test.outcome {
                Some(Outcome::Passed) => passed_tests.push(test.test),
                Some(Outcome::Failed { msg }) => failed_tests.push((test.test, msg)),
                Some(Outcome::Measured { average, variance }) => {
                    benchmark_tests.push((test.test, average, variance))
                }
                None => ignored_tests.push(test.test),
            }
        }

        let status = if failed_tests.is_empty() {
            console::style("ok").green()
        } else {
            console::style("FAILED").red()
        };

        println!();
        println!("test result: {status}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured",
            status = status,
            passed = passed_tests.len(),
            failed = failed_tests.len(),
            ignored = ignored_tests.len(),
            measured = benchmark_tests.len(),
        );

        if failed_tests.is_empty() {
            0
        } else {
            crate::ERROR_STATUS_CODE
        }
    }
}
