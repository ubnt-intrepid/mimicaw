use futures::future::{join_all, Future};
use mimi::{TestOptions, TestSuite};
use tokio::task::{self, JoinHandle};

#[derive(Default)]
struct JobServer(Vec<JoinHandle<()>>);

impl JobServer {
    fn spawn<Fut>(&mut self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.0.push(task::spawn(future));
    }

    async fn wait_all(&mut self) {
        join_all(self.0.drain(..)).await;
    }
}

#[tokio::main]
async fn main() {
    let report = {
        let mut tests = TestSuite::from_env();
        let mut jobs = JobServer::default();

        if let Some(test) = tests.add_test("case1", TestOptions::default()) {
            jobs.spawn(test.run(async {
                // do stuff...
                Ok(())
            }));
        }

        if let Some(test) = tests.add_test("case2", TestOptions::default()) {
            jobs.spawn(test.run(async {
                // do stuff...
                Err(Some("foo".into()))
            }));
        }

        if let Some(test) = tests.add_test("case3", TestOptions::ignored()) {
            jobs.spawn(test.run(async move {
                // do stuff ...
                Ok(())
            }));
        }

        tests.run_tests(jobs.wait_all()).await
    };

    println!("{:?}", report);
    report.exit()
}
