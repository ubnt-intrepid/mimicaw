use mimi::{TestDriver, TestOptions};
use tokio::task;

#[tokio::main]
async fn main() {
    std::process::exit({
        let mut driver = TestDriver::from_env();

        if let Some(test) = driver.add_test("case1", TestOptions::test()) {
            task::spawn(test.run_test(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
                // do stuff...
                Ok(())
            }));
        }

        if let Some(test) = driver.add_test("case2", TestOptions::test()) {
            task::spawn(test.run_test(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(6)).await;
                // do stuff...
                Err(Some("foo".into()))
            }));
        }

        if let Some(test) =
            driver.add_test("case3_a_should_be_zero", TestOptions::test().ignored(true))
        {
            task::spawn(test.run_test(async move {
                tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;
                // do stuff ...
                Ok(())
            }));
        }

        driver.run_tests().await
    });
}
