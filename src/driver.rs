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
use pin_utils::pin_mut;
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

pin_project! {
    struct PendingTests<'a, D, R> {
        #[pin]
        inner: Vec<PendingTest<'a, D, R>>,
    }
}

impl<'a, D, R> PendingTests<'a, D, R> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &PendingTest<'a, D, R>> + '_ {
        self.inner.iter()
    }

    fn iter_pin_mut(
        self: Pin<&mut Self>,
    ) -> impl Iterator<Item = Pin<&mut PendingTest<'a, D, R>>> + '_ {
        let me = self.project();

        // Safety:
        // * The mutable borrow take out here is used only for creating `std::slice::IterMut`,
        //   and the addition or deletion of element(s) never occurs.
        // * `IterMut` does not move the element during scanning.
        #[allow(unsafe_code)]
        unsafe {
            let inner = me.inner.get_unchecked_mut();
            inner.iter_mut().map(|test| Pin::new_unchecked(test))
        }
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

        let (pending_tests, num_filtered_out) = {
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

            (
                PendingTests {
                    inner: pending_tests,
                },
                num_filtered_out,
            )
        };

        pin_mut!(pending_tests);

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

        futures_util::stream::iter(pending_tests.as_mut().iter_pin_mut()) //
            .for_each_concurrent(None, |mut test| {
                test.as_mut()
                    .start(&self.args, max_name_length, &mut runner);
                test
            })
            .await;

        let mut num_passed = 0;
        let mut failed_tests = vec![];
        let mut num_measured = 0;
        let mut num_ignored = 0;
        for test in pending_tests.iter() {
            match test.outcome {
                Some(ref outcome) => match outcome.kind() {
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
            let _ = writeln!(self.printer.term(), "failures:");
            for (desc, msg) in &failed_tests {
                let _ = writeln!(self.printer.term(), "---- {} ----", desc.name());
                if let Some(msg) = msg {
                    let _ = self.printer.term().write_str(&*msg);
                    if msg.chars().last().map_or(true, |c| c != '\n') {
                        let _ = self.printer.term().write_str("\n");
                    }
                }
            }

            let _ = writeln!(self.printer.term());
            let _ = writeln!(self.printer.term(), "failures:");
            for (desc, _) in &failed_tests {
                let _ = writeln!(self.printer.term(), "    {}", desc.name());
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
