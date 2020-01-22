use async_std::task;
use futures_timer::Delay;
use mimicaw::Test;
use std::time::Duration;

#[async_std::main]
async fn main() {
    let tests = vec![
        Test::test("case1", async {
            task::spawn(async {
                Delay::new(Duration::from_secs(8)).await;
                // do stuff...
                Ok(())
            })
            .await
        }),
        Test::test("case2", async {
            task::spawn(async {
                Delay::new(Duration::from_secs(4)).await;
                // do stuff...
                Err(Some("foo".into()))
            })
            .await
        }),
        Test::test("case3", async {
            task::spawn(async move {
                Delay::new(Duration::from_secs(6)).await;
                // do stuff ...
                Ok(())
            })
            .await
        })
        .ignored(true),
    ];

    let status = mimicaw::run_tests(tests).await;
    std::process::exit(status);
}
