use futures::{executor::LocalPool, task::LocalSpawnExt};
use mimi::{TestOptions, TestSuite};

fn main() {
    std::process::exit({
        let mut tests = TestSuite::from_env();
        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        if let Some(bench) = tests.add_bench("bench1", TestOptions::new()) {
            spawner
                .spawn_local(bench.run(async {
                    // do stuff...
                    Ok((1274, 23))
                }))
                .unwrap();
        }

        if let Some(bench) = tests.add_bench("bench2", TestOptions::new().ignored(true)) {
            spawner
                .spawn_local(bench.run(async {
                    // do stuff...
                    Ok((23, 430))
                }))
                .unwrap();
        }

        if let Some(test) = tests.add_test("test1", TestOptions::new()) {
            spawner
                .spawn_local(test.run(async {
                    // do stuff...
                    Ok(())
                }))
                .unwrap();
        }

        if let Some(test) = tests.add_test("test2", TestOptions::new().ignored(true)) {
            spawner
                .spawn_local(test.run(async {
                    // do stuff...
                    Ok(())
                }))
                .unwrap();
        }

        pool.run_until(tests.run_tests())
    });
}
