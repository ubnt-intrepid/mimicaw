use futures::{executor::block_on, future::Future};
use mimi::{TestOptions, TestSuite};
use std::pin::Pin;

fn main() {
    std::process::exit(block_on(async {
        let mut tests = TestSuite::from_env();
        let mut jobs = Vec::<Pin<Box<dyn Future<Output = ()>>>>::new();

        if let Some(bench) = tests.add_bench("bench1", TestOptions::new()) {
            jobs.push(Box::pin(bench.run(async {
                // do stuff...
                Ok((1274, 23))
            })));
        }

        if let Some(bench) = tests.add_bench("bench2", TestOptions::new().ignored(true)) {
            jobs.push(Box::pin(bench.run(async {
                // do stuff...
                Ok((23, 430))
            })));
        }

        if let Some(test) = tests.add_test("test1", TestOptions::new()) {
            jobs.push(Box::pin(test.run(async {
                // do stuff...
                Ok(())
            })));
        }

        if let Some(test) = tests.add_test("test2", TestOptions::new().ignored(true)) {
            jobs.push(Box::pin(test.run(async {
                // do stuff...
                Ok(())
            })));
        }

        tests
            .run_tests(async {
                futures::future::join_all(jobs).await;
            })
            .await
    }));
}
