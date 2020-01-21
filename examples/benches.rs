use futures::executor::block_on;
use futures_timer::Delay;
use mimi::{Test, TestDriver};
use std::time::Duration;

fn main() {
    std::process::exit(block_on(async {
        let mut driver = TestDriver::from_env();

        driver.add_test(Test::bench("bench1", async {
            // do stuff...
            Delay::new(Duration::from_secs(4)).await;
            Ok((1274, 23))
        }));

        driver.add_test(
            Test::bench("bench2", async {
                // do stuff...
                Delay::new(Duration::from_secs(8)).await;
                Ok((23, 430))
            })
            .ignored(true),
        );

        driver.add_test(Test::test("test1", async {
            // do stuff...
            Delay::new(Duration::from_secs(2)).await;
            Ok(())
        }));

        driver.add_test(
            Test::test("test2", async {
                // do stuff...
                Delay::new(Duration::from_secs(6)).await;
                Ok(())
            })
            .ignored(true),
        );

        driver.run_tests().await
    }));
}
