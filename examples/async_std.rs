use async_std::task;
use futures_timer::Delay;
use mimi::{TestDriver, TestOptions};
use std::time::Duration;

#[async_std::main]
async fn main() {
    std::process::exit({
        let mut driver = TestDriver::from_env();

        if let Some(test) = driver.add_test("case1", TestOptions::test()) {
            task::spawn(test.run_test(async {
                Delay::new(Duration::from_secs(8)).await;
                // do stuff...
                Ok(())
            }));
        }

        if let Some(test) = driver.add_test("case2", TestOptions::test()) {
            task::spawn(test.run_test(async {
                Delay::new(Duration::from_secs(4)).await;
                // do stuff...
                Err(Some("foo".into()))
            }));
        }

        if let Some(test) = driver.add_test("case3", TestOptions::test().ignored(true)) {
            task::spawn(test.run_test(async move {
                Delay::new(Duration::from_secs(6)).await;
                // do stuff ...
                Ok(())
            }));
        }

        driver.run_tests().await
    });
}
