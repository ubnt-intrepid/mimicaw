use async_std::task;
use mimi::{TestOptions, TestSuite};

#[async_std::main]
async fn main() {
    std::process::exit({
        let mut tests = TestSuite::from_env();

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

        tests.run_tests().await
    });
}
