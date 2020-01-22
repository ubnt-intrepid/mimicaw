use futures::executor::block_on;
use futures_timer::Delay;
use mimicaw::{Outcome, Test};
use rand::{
    distributions::{Bernoulli, Distribution},
    seq::IteratorRandom,
};
use std::time::Duration;

fn main() {
    let mut rng = rand::thread_rng();
    let bernoulli = Bernoulli::new(0.8).unwrap();
    let intervals = &[2, 3, 5, 7];

    let tests = (0..20).map(|i| {
        let name = format!("test-{:03}", i);
        let interval = intervals.iter().choose(&mut rng).copied().unwrap_or(1);
        let delay = Delay::new(Duration::from_secs(interval));
        let outcome = if bernoulli.sample(&mut rng) {
            Outcome::passed()
        } else {
            Outcome::failed()
        };
        Test::test(&name, (delay, outcome))
    });

    let status = block_on(mimicaw::run_tests(tests, |_desc, (delay, outcome)| {
        Box::pin(async move {
            delay.await;
            outcome
        })
    }));
    std::process::exit(status);
}
