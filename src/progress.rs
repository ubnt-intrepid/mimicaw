use crate::test::{Outcome, OutcomeKind};
use futures::channel::oneshot;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use num_format::{Locale, ToFormattedString};

pub(crate) struct Container {
    progress: MultiProgress,
    style: ProgressStyle,
}

impl Container {
    pub(crate) fn new(name_width: usize) -> Self {
        let progress = MultiProgress::new();
        let style = ProgressStyle::default_spinner() //
            .template(&format!(
                "{{spinner}} test {{prefix:{}}} ... {{wide_msg}}",
                name_width,
            ));
        Self { progress, style }
    }

    pub(crate) fn add_progress(&self, name: &str) -> Progress {
        let progress_bar = self.progress.add(ProgressBar::new_spinner());
        progress_bar.set_prefix(name);
        progress_bar.set_style(self.style.clone());
        Progress { bar: progress_bar }
    }

    pub(crate) async fn join(self) {
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let res = self.progress.join();
            let _ = tx.send(res);
        });
        rx.await.unwrap().unwrap()
    }
}

pub(crate) struct Progress {
    bar: ProgressBar,
}

impl Progress {
    pub(crate) fn set_running(&self) {
        self.bar.enable_steady_tick(100);
    }

    pub(crate) fn finish(&self, outcome: Option<&Outcome>) {
        self.bar
            .finish_with_message(&match outcome.as_ref().map(|outcome| outcome.kind()) {
                Some(OutcomeKind::Passed) => console::style("ok").green().to_string(),
                Some(OutcomeKind::Failed { .. }) => console::style("FAILED").red().to_string(),
                Some(OutcomeKind::Measured { average, variance }) => format!(
                    "{}: {:>10} ns/iter (+/- {})",
                    console::style("bench").cyan(),
                    average.to_formatted_string(&Locale::en),
                    variance
                ),
                None => console::style("ignored").yellow().to_string(),
            });
    }
}
