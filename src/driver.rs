use crate::{
    args::Args,
    printer::Printer,
    report::Report,
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
    type Future: Future<Output = Outcome>;

    /// Run a test case.
    fn run(&mut self, desc: TestDesc, data: D) -> Self::Future;
}

impl<F, D, R> TestRunner<D> for F
where
    F: FnMut(TestDesc, D) -> R,
    R: Future<Output = Outcome>,
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

impl<D, R> PendingTest<'_, D, R> {
    fn start<F>(self: Pin<&mut Self>, args: &Args, name_length: usize, runner: &mut F)
    where
        F: TestRunner<D, Future = R>,
        R: Future<Output = Outcome>,
    {
        let mut me = self.project();

        *me.name_length = name_length;

        let ignored = (me.desc.ignored() && !args.run_ignored)
            || match me.desc.kind() {
                TestKind::Test => !args.run_tests,
                TestKind::Bench => !args.run_benchmarks,
            };

        let context = me
            .context
            .take()
            .expect("the context has already been used");

        if !ignored {
            let test_case = runner.run(me.desc.clone(), context);
            me.test_case.set(Some(test_case));
        }
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
    ) -> Result<Report, ExitStatus> {
        let mut runner = runner;

        // First, convert each test case to PendingTest for tracking the running state.
        // Test cases that satisfy the skip condition are filtered out here.
        let mut pending_tests = vec![];
        let mut filtered_out_tests = vec![];
        let mut unique_test_names = HashSet::new();
        for test in tests {
            if !unique_test_names.insert(test.desc().name().to_string()) {
                let _ = writeln!(
                    self.printer.term(),
                    "the test name is conflicted: {}",
                    test.desc().name()
                );
                return Err(ExitStatus::FAILED);
            }

            if self.args.is_filtered(test.desc().name()) {
                filtered_out_tests.push(test);
                continue;
            }

            // Since PendingTest may contain the immovable state must be pinned
            // before starting any operations.
            // Here, each test case is allocated on the heap.
            let (desc, context) = test.deconstruct();
            pending_tests.push(Box::pin(PendingTest {
                desc,
                context: Some(context),
                test_case: None,
                outcome: None,
                printer: &self.printer,
                name_length: 0,
            }));
        }

        if self.args.list {
            self.printer
                .print_list(pending_tests.iter().map(|test| &test.desc));
            return Err(ExitStatus::OK);
        }

        let _ = writeln!(self.printer.term(), "running {} tests", pending_tests.len());

        let max_name_length = pending_tests
            .iter()
            .map(|test| test.desc.name().len())
            .max()
            .unwrap_or(0);

        futures_util::stream::iter(pending_tests.iter_mut()) //
            .for_each_concurrent(None, |test| {
                test.as_mut()
                    .start(&self.args, max_name_length, &mut runner);
                test
            })
            .await;

        let mut passed = vec![];
        let mut failed = vec![];
        let mut measured = vec![];
        let mut ignored = vec![];
        for test in &pending_tests {
            match test.outcome {
                Some(ref outcome) => match outcome.kind() {
                    OutcomeKind::Passed => passed.push(test.desc.clone()),
                    OutcomeKind::Failed => failed.push((test.desc.clone(), outcome.err_msg())),
                    OutcomeKind::Measured { average, variance } => {
                        measured.push((test.desc.clone(), (*average, *variance)))
                    }
                },
                None => ignored.push(test.desc.clone()),
            }
        }

        let report = Report {
            passed,
            failed,
            measured,
            ignored,
            filtered_out: filtered_out_tests
                .into_iter()
                .map(|test| {
                    let (desc, _) = test.deconstruct();
                    desc
                })
                .collect(),
        };
        let _ = report.print(&self.printer);

        Ok(report)
    }
}
