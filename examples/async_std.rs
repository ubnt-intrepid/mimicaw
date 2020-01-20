use async_std::task;
use mimi::{TestDriver, TestOptions};

#[async_std::main]
async fn main() {
    std::process::exit({
        let mut driver = TestDriver::from_env();

        {
            let mut tests = driver.test_suite();

            if let Some(test) = tests.add_test("case1", TestOptions::new()) {
                task::spawn(test.run(async {
                    // do stuff...
                    Ok(())
                }));
            }

            if let Some(test) = tests.add_test("case2", TestOptions::new()) {
                task::spawn(test.run(async {
                    // do stuff...
                    Err(Some("foo".into()))
                }));
            }

            if let Some(test) = tests.add_test("case3", TestOptions::new().ignored(true)) {
                task::spawn(test.run(async move {
                    // do stuff ...
                    Ok(())
                }));
            }
        }

        driver.run_tests().await
    });
}
