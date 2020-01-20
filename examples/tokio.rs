use mimi::{TestOptions, TestSuite};
use tokio::task;

#[tokio::main]
async fn main() {
    std::process::exit({
        let mut tests = TestSuite::from_env();

        if let Some(test) = tests.add_test("case1", TestOptions::new()) {
            task::spawn(test.run(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
                // do stuff...
                Ok(())
            }));
        }

        if let Some(test) = tests.add_test("case2", TestOptions::new()) {
            task::spawn(test.run(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(6)).await;
                // do stuff...
                Err(Some("foo".into()))
            }));
        }

        if let Some(test) = tests.add_test("case3", TestOptions::new().ignored(true)) {
            task::spawn(test.run(async move {
                tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;
                // do stuff ...
                Ok(())
            }));
        }

        tests.run_tests().await
    });
}
