use crate::{
    args::Args,
    test::{Outcome, Test, TestKind},
};
use futures::{
    channel::oneshot,
    future::Future,
    stream::StreamExt,
    task::{self, Poll},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pin_project_lite::pin_project;
use std::{collections::HashSet, pin::Pin};

pin_project! {
    struct RunningTest {
        #[pin]
        test: Test,
        filtered: bool,
        progress: Option<ProgressBar>,
        outcome: Option<Outcome>,
    }
}

impl Future for RunningTest {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let progress = me.progress.as_ref().expect("progress bar is not set");
        if *me.filtered {
            progress.finish_with_message(&console::style("ignored").yellow().to_string());
            return Poll::Ready(());
        }

        progress.enable_steady_tick(100);
        progress.set_message("running");

        let outcome = futures::ready!(me.test.test_case().poll(cx));
        progress.finish_with_message(&match outcome {
            Outcome::Passed => console::style("passed").green().to_string(),
            Outcome::Failed { .. } => console::style("failed").red().to_string(),
            Outcome::Measured { .. } => console::style("measured").green().to_string(),
        });
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

    fn print_list(&self, tests: impl IntoIterator<Item = Test>) {
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
    pub async fn run_tests<I>(&mut self, tests: I) -> i32
    where
        I: IntoIterator<Item = Test>,
    {
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
            let filtered = test.ignored || !is_target_mode || self.args.is_filtered(&*test.name());
            let filtered = filtered ^ self.args.run_ignored;

            running_tests.push(RunningTest {
                test,
                filtered,
                progress: None,
                outcome: None,
            });
        }

        println!("======== RUNNING TESTS ========");
        let multi_progress = MultiProgress::new();
        let progress_style = ProgressStyle::default_spinner() //
            .template(&format!(
                "{{prefix:{}.bold.dim}} {{spinner}} {{wide_msg}}",
                max_name_length,
            ));
        running_tests.iter_mut().for_each(|test| {
            let progress = multi_progress.add(ProgressBar::new_spinner());
            progress.set_prefix(&*test.test.name());
            progress.set_style(progress_style.clone());
            test.progress.replace(progress);
        });

        let run_tests = futures::stream::iter(running_tests.iter_mut()) //
            .for_each_concurrent(1024, std::convert::identity);

        let complete_progress = {
            let (tx, rx) = oneshot::channel();
            std::thread::spawn(move || {
                let res = multi_progress.join();
                let _ = tx.send(res);
            });
            async move { rx.await.unwrap().unwrap() }
        };
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

        println!();
        println!("======== SUMMARY ========");
        println!("PASSED: {}", passed_tests.len());
        println!("FAILED: {}", failed_tests.len());
        println!("IGNORE: {}", ignored_tests.len());
        println!("BENCH:  {}", benchmark_tests.len());

        if failed_tests.is_empty() {
            0
        } else {
            crate::ERROR_STATUS_CODE
        }
    }
}
