use crate::{
    args::{Args, OutputFormat},
    test::{Outcome, OutcomeKind, TestDesc, TestKind},
};
use console::Term;
use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) struct Printer {
    term: Term,
    name_length: AtomicUsize,
    format: OutputFormat,
}

impl Printer {
    pub(crate) fn new(args: &Args) -> Self {
        Self {
            term: Term::buffered_stdout(),
            name_length: AtomicUsize::new(0),
            format: args.format,
        }
    }

    pub(crate) fn term(&self) -> &Term {
        &self.term
    }

    pub(crate) fn set_name_length(&self, value: usize) {
        self.name_length.store(value, Ordering::SeqCst);
    }

    pub(crate) fn print_list(&self, tests: impl IntoIterator<Item = impl AsRef<TestDesc>>) {
        let quiet = self.format == crate::args::OutputFormat::Terse;

        let mut num_tests = 0;
        let mut num_benches = 0;

        for test in tests {
            let desc = test.as_ref();
            let kind_str = match desc.kind() {
                TestKind::Test => {
                    num_tests += 1;
                    "test"
                }
                TestKind::Bench => {
                    num_benches += 1;
                    "benchmark"
                }
            };
            let _ = writeln!(&mut &self.term, "{}: {}", desc.name(), kind_str);
        }

        if !quiet {
            fn plural_suffix(n: usize) -> &'static str {
                match n {
                    1 => "",
                    _ => "s",
                }
            }

            if num_tests != 0 || num_benches != 0 {
                let _ = writeln!(&mut &self.term);
            }
            let _ = writeln!(
                &mut &self.term,
                "{} test{}, {} benchmark{}",
                num_tests,
                plural_suffix(num_tests),
                num_benches,
                plural_suffix(num_benches)
            );
        }
    }

    pub(crate) fn print_result(&self, desc: &TestDesc, outcome: Option<&Outcome>) {
        match self.format {
            OutputFormat::Pretty => self.print_result_pretty(desc, outcome),
            OutputFormat::Terse => self.print_result_terse(desc, outcome),
            OutputFormat::Json => eprintln!(
                "{warning}: JSON format is not supported",
                warning = console::style("warning").yellow()
            ),
        }
    }

    fn print_result_pretty(&self, desc: &TestDesc, outcome: Option<&Outcome>) {
        let name = desc.name();
        let name_length = self.name_length.load(Ordering::SeqCst);

        match outcome {
            Some(outcome) => match outcome.kind() {
                OutcomeKind::Passed => {
                    let _ = writeln!(
                        &mut &self.term,
                        "test {0:<1$} ... {2}",
                        name,
                        name_length,
                        console::style("ok").green()
                    );
                }
                OutcomeKind::Failed => {
                    let _ = writeln!(
                        &mut &self.term,
                        "test {0:<1$} ... {2}",
                        name,
                        name_length,
                        console::style("FAILED").red()
                    );
                }
                OutcomeKind::Measured { average, variance } => {
                    let _ = writeln!(
                        &mut &self.term,
                        "test {0:<1$} ... {2}: {3:>11} ns/iter (+/- {4})",
                        name,
                        name_length,
                        console::style("bench").cyan(),
                        average,
                        variance
                    );
                }
            },
            None => {
                let _ = writeln!(
                    &mut &self.term,
                    "test {0:<1$} ... {2}",
                    name,
                    name_length,
                    console::style("ignored").yellow()
                );
            }
        }
        let _ = self.term.flush();
    }

    fn print_result_terse(&self, desc: &TestDesc, outcome: Option<&Outcome>) {
        let ch = match outcome {
            Some(o) => match o.kind() {
                OutcomeKind::Passed => ".",
                OutcomeKind::Failed => "F",
                OutcomeKind::Measured { .. } => {
                    // benchmark test does not support terse format.
                    return self.print_result_pretty(desc, outcome);
                }
            },
            None => "i",
        };
        let _ = self.term.write_str(ch);
        let _ = self.term.flush();
    }
}
