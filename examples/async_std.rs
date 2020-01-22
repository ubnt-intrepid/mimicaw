use async_std::task;
use futures::future::Future;
use futures_timer::Delay;
use mimicaw::{Outcome, Test};
use std::{pin::Pin, time::Duration};

#[async_std::main]
async fn main() {
    let tests = vec![
        Test::test("case1").context(Box::pin(async {
            task::spawn(async {
                Delay::new(Duration::from_secs(8)).await;
                // do stuff...
                Outcome::passed()
            })
            .await
        }) as Pin<Box<dyn Future<Output = Outcome>>>),
        Test::test("case2").context(Box::pin(async {
            task::spawn(async {
                Delay::new(Duration::from_secs(4)).await;
                // do stuff...
                Outcome::failed().error_message("foo")
            })
            .await
        })),
        Test::test("case3").ignore(true).context(Box::pin(async {
            task::spawn(async move {
                Delay::new(Duration::from_secs(6)).await;
                // do stuff ...
                Outcome::passed()
            })
            .await
        })),
    ];

    let status = mimicaw::run_tests(tests, std::convert::identity).await;
    std::process::exit(status);
}
