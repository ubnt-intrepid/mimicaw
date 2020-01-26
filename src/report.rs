use crate::{printer::Printer, test::TestDesc, ExitStatus};
use std::{
    borrow::Cow,
    io::{self, Write},
    sync::Arc,
};

/// A report on test suite execution.
#[derive(Debug)]
#[non_exhaustive]
pub struct Report {
    /// Passed test cases.
    pub passed: Vec<TestDesc>,

    /// Failed test cases with the error messages.
    pub failed: Vec<(TestDesc, Option<Arc<Cow<'static, str>>>)>,

    /// Benchmark results.
    pub measured: Vec<(TestDesc, (u64, u64))>,

    /// Test cases skipped because they do not satisfy the execution conditions.
    pub ignored: Vec<TestDesc>,

    /// Test cases filtered out.
    pub filtered_out: Vec<TestDesc>,
}

impl Report {
    /// Return an exit status used as a result of the test process.
    pub fn status(&self) -> ExitStatus {
        if self.failed.is_empty() {
            ExitStatus::OK
        } else {
            ExitStatus::FAILED
        }
    }

    /// Return an iterator of skipped test cases.
    #[inline]
    pub fn skipped(&self) -> impl Iterator<Item = (&TestDesc, &str)> + '_ {
        let ignored = self.ignored.iter().map(|desc| (desc, "ignored"));
        let filtered_out = self.filtered_out.iter().map(|desc| (desc, "filtered out"));
        ignored.chain(filtered_out)
    }

    pub(crate) fn print(&self, printer: &Printer) -> io::Result<()> {
        let mut status = printer.styled("ok").green();

        if !self.failed.is_empty() {
            status = printer.styled("FAILED").red();
            writeln!(printer.term())?;
            writeln!(printer.term(), "failures:")?;
            for (desc, msg) in &self.failed {
                writeln!(printer.term(), "---- {} ----", desc.name())?;
                if let Some(msg) = msg {
                    write!(printer.term(), "{}", msg)?;
                    if msg.chars().last().map_or(true, |c| c != '\n') {
                        writeln!(printer.term())?;
                    }
                }
            }

            writeln!(printer.term())?;
            writeln!(printer.term(), "failures:")?;
            for (desc, _) in &self.failed {
                writeln!(printer.term(), "    {}", desc.name())?;
            }
        }

        writeln!(printer.term())?;
        writeln!(printer.term(), "test result: {status}. {passed} passed; {failed} failed; {ignored} ignored; {measured} measured; {filtered_out} filtered out",
            status = status,
            passed = self.passed.len(),
            failed = self.failed.len(),
            ignored = self.ignored.len(),
            measured = self.measured.len(),
            filtered_out = self.filtered_out.len(),
        )?;

        Ok(())
    }
}
