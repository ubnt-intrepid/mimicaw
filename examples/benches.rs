use futures::{executor::LocalPool, task::LocalSpawnExt};
use futures_timer::Delay;
use mimi::{TestDriver, TestOptions};
use std::time::Duration;

fn main() {
    std::process::exit({
        let mut driver = TestDriver::from_env();
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        if let Some(bench) = driver.add_bench("bench1", TestOptions::new()) {
            spawner
                .spawn_local(bench.run(async {
                    // do stuff...
                    Delay::new(Duration::from_secs(4)).await;
                    Ok((1274, 23))
                }))
                .unwrap();
        }

        if let Some(bench) = driver.add_bench("bench2", TestOptions::new().ignored(true)) {
            spawner
                .spawn_local(bench.run(async {
                    // do stuff...
                    Delay::new(Duration::from_secs(8)).await;
                    Ok((23, 430))
                }))
                .unwrap();
        }

        if let Some(test) = driver.add_test("test1", TestOptions::new()) {
            spawner
                .spawn_local(test.run(async {
                    // do stuff...
                    Delay::new(Duration::from_secs(2)).await;
                    Ok(())
                }))
                .unwrap();
        }

        if let Some(test) = driver.add_test("test2", TestOptions::new().ignored(true)) {
            spawner
                .spawn_local(test.run(async {
                    // do stuff...
                    Delay::new(Duration::from_secs(6)).await;
                    Ok(())
                }))
                .unwrap();
        }

        pool.run_until(driver.run_tests())
    });
}
