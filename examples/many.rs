use futures::executor::block_on;
use futures_timer::Delay;
use mimicaw::{Args, Outcome, Test};
use rand::distributions::{Bernoulli, Distribution, Uniform};
use std::time::Duration;

fn main() {
    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let tests = (0..50).map(|i| Test::test(format!("test-{:03}", i), ()));

    let mut rng = rand::thread_rng();
    let bernoulli = Bernoulli::new(0.8).unwrap();
    let uniform = Uniform::from(500..=5000);

    block_on(mimicaw::run_tests(&args, tests, |_desc, ()| {
        let interval = uniform.sample(&mut rng);
        let passed = bernoulli.sample(&mut rng);
        async move {
            Delay::new(Duration::from_millis(interval)).await;
            if passed {
                Outcome::passed()
            } else {
                Outcome::failed()
            }
        }
    }))
    .exit();
}
