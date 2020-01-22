use async_std::task;
use futures::prelude::*;
use futures_timer::Delay;
use mimicaw::{Outcome, Test};
use std::time::Duration;

#[async_std::main]
async fn main() {
    let tests = vec![
        Test::test("case1", {
            async {
                task::spawn(async {
                    Delay::new(Duration::from_secs(8)).await;
                    // do stuff...
                    Outcome::passed()
                })
                .await
            }
            .boxed_local()
        }),
        Test::test("case2", {
            async {
                task::spawn(async {
                    Delay::new(Duration::from_secs(4)).await;
                    // do stuff...
                    Outcome::failed().error_message("foo")
                })
                .await
            }
            .boxed_local()
        }),
        Test::test("case3", {
            async {
                task::spawn(async move {
                    Delay::new(Duration::from_secs(6)).await;
                    // do stuff ...
                    Outcome::passed()
                })
                .await
            }
            .boxed_local()
        }),
    ];

    let status = mimicaw::run_tests(tests, |_, fut| fut).await;
    std::process::exit(status);
}
