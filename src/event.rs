#[derive(Debug)]
#[non_exhaustive]
pub enum Outcome {
    Passed,
    Failed {
        msg: Option<String>,
    },
    Ignored,
    Measured {
        average: u64,
        variance: u64,
    },

    #[doc(hidden)]
    Canceled,
}

#[derive(Debug)]
#[must_use]
pub struct Report {
    pub(crate) has_failed: bool,
}

impl Report {
    pub fn has_failed(&self) -> bool {
        self.has_failed
    }

    pub fn exit(self) -> ! {
        if self.has_failed {
            std::process::exit(101);
        }
        std::process::exit(0);
    }
}

pub trait EventHandler {
    fn list_tests(&self, tests: &[String]);

    fn dump_result(&self, name: &str, outcome: Outcome);

    fn dump_summary(&self, report: &Report);
}

#[derive(Debug, Default)]
pub struct DefaultEventHandler(());

impl EventHandler for DefaultEventHandler {
    fn list_tests(&self, _tests: &[String]) {}

    fn dump_result(&self, name: &str, outcome: Outcome) {
        match outcome {
            Outcome::Passed => println!("{}: passed", name),
            Outcome::Ignored => println!("{}: ignored", name),
            Outcome::Canceled => println!("{}: canceled", name),
            Outcome::Measured { average, variance } => {
                println!("{}: measured (avg={}, var={})", name, average, variance)
            }
            Outcome::Failed { msg } => match msg {
                Some(msg) => println!("{}: failed:\n{}", name, msg),
                None => println!("{}: failed", name),
            },
        }
    }

    fn dump_summary(&self, _report: &Report) {}
}
