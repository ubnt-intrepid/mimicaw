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

pub trait EventHandler {
    fn dump_result(&self, name: &str, outcome: Outcome);
}

#[derive(Debug, Default)]
pub struct DefaultEventHandler(());

impl EventHandler for DefaultEventHandler {
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
}
