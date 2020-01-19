use futures::{executor::block_on, future::Future};
use mimi::{Measurement, TestOptions, TestSuite};
use std::pin::Pin;

fn main() {
    let report = block_on(async {
        let mut runner = TestSuite::from_env();
        let mut jobs = Vec::<Pin<Box<dyn Future<Output = ()>>>>::new();

        if let Some(bench) = runner.add_bench("bench1", TestOptions::default()) {
            jobs.push(Box::pin(bench.run(async {
                // do stuff...
                Ok(Measurement {
                    average: 1274,
                    variance: 23,
                })
            })));
        }

        if let Some(bench) = runner.add_bench("bench2", TestOptions::ignored()) {
            jobs.push(Box::pin(bench.run(async {
                // do stuff...
                Ok(Measurement {
                    average: 23,
                    variance: 430,
                })
            })));
        }

        if let Some(test) = runner.add_test("test1", TestOptions::default()) {
            jobs.push(Box::pin(test.run(async {
                // do stuff...
                Ok(())
            })));
        }

        if let Some(test) = runner.add_test("test2", TestOptions::ignored()) {
            jobs.push(Box::pin(test.run(async {
                // do stuff...
                Ok(())
            })));
        }

        runner
            .run_tests(async {
                futures::future::join_all(jobs).await;
            })
            .await
    });

    println!("{:?}", report);
    report.exit()
}
