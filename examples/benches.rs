use futures::{executor::block_on, future::Future};
use futures_timer::Delay;
use mimicaw::{Outcome, Test};
use std::{pin::Pin, time::Duration};

fn main() {
    let tests = vec![
        Test::bench("bench1").context(Box::pin(async {
            // do stuff...
            Delay::new(Duration::from_secs(4)).await;
            Outcome::Measured {
                average: 1274,
                variance: 23,
            }
        }) as Pin<Box<dyn Future<Output = Outcome>>>),
        Test::bench("bench2").ignore(true).context(Box::pin(async {
            // do stuff...
            Delay::new(Duration::from_secs(8)).await;
            Outcome::Measured {
                average: 23,
                variance: 430,
            }
        })),
        Test::test("test1").context(Box::pin(async {
            // do stuff...
            Delay::new(Duration::from_secs(2)).await;
            Outcome::Passed
        })),
        Test::test("test2") //
            .ignore(true)
            .context(Box::pin(async {
                // do stuff...
                Delay::new(Duration::from_secs(6)).await;
                Outcome::Passed
            })),
    ];

    let status = block_on(mimicaw::run_tests(tests, std::convert::identity));
    std::process::exit(status);
}
