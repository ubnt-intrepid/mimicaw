use mimi::TestSuite;

#[derive(Default)]
struct JobServer(Vec<tokio::task::JoinHandle<()>>);

impl JobServer {
    fn spawn<Fut>(&mut self, future: Fut)
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.0.push(tokio::task::spawn(future));
    }

    async fn wait_all(&mut self) {
        futures::future::join_all(self.0.drain(..)).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut runner = TestSuite::from_env();
    let mut jobs = JobServer::default();

    if let Some(test) = runner.add_test("case1", false) {
        jobs.spawn(test.run(async {
            // do stuff...
            Ok(())
        }));
    }

    if let Some(test) = runner.add_test("case2", false) {
        jobs.spawn(test.run(async {
            // do stuff...
            Err(Some("foo".into()))
        }));
    }

    if let Some(test) = runner.add_test("case3", true) {
        jobs.spawn(test.run(async move {
            // do stuff ...
            Ok(())
        }));
    }

    let report = runner
        .run_tests(async {
            jobs.wait_all().await;
        })
        .await;
    println!("{:?}", report);
    report.exit()
}
