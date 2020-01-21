use async_std::task;
use futures_timer::Delay;
use mimi::{Test, TestDriver};
use std::time::Duration;

#[async_std::main]
async fn main() {
    std::process::exit({
        let mut driver = TestDriver::from_env();

        driver.add_test(Test::test("case1", async {
            task::spawn(async {
                Delay::new(Duration::from_secs(8)).await;
                // do stuff...
                Ok(())
            })
            .await
        }));

        driver.add_test(Test::test("case2", async {
            task::spawn(async {
                Delay::new(Duration::from_secs(4)).await;
                // do stuff...
                Err(Some("foo".into()))
            })
            .await
        }));

        driver.add_test(
            Test::test("case3", async {
                task::spawn(async move {
                    Delay::new(Duration::from_secs(6)).await;
                    // do stuff ...
                    Ok(())
                })
                .await
            })
            .ignored(true),
        );

        driver.run_tests().await
    });
}
