use futures::{executor::block_on, future::Future};
use futures_timer::Delay;
use mimicaw::{Outcome, Test};
use std::{pin::Pin, time::Duration};

fn main() {
    let tests = vec![
        Test::bench("bench1").context(Box::pin(async {
            // do stuff...
            Delay::new(Duration::from_secs(4)).await;
            Outcome::measured(1274, 23)
        }) as Pin<Box<dyn Future<Output = Outcome>>>),
        Test::bench("bench2").ignore(true).context(Box::pin(async {
            // do stuff...
            Delay::new(Duration::from_secs(8)).await;
            Outcome::measured(23, 430)
        })),
        Test::test("test1").context(Box::pin(async {
            // do stuff...
            Delay::new(Duration::from_secs(2)).await;
            Outcome::passed()
        })),
        Test::test("test2") //
            .ignore(true)
            .context(Box::pin(async {
                // do stuff...
                Delay::new(Duration::from_secs(6)).await;
                Outcome::passed()
            })),
    ];

    let status = block_on(mimicaw::run_tests(tests, std::convert::identity));
    std::process::exit(status);
}
