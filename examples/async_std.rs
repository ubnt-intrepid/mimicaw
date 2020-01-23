use async_std::task;
use futures_timer::Delay;
use mimicaw::{Args, Outcome, Test};
use std::time::Duration;

type TestFn = fn() -> task::JoinHandle<Outcome>;

#[async_std::main]
async fn main() {
    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let tests: Vec<Test<TestFn>> = vec![
        Test::test("case1", || {
            task::spawn(async {
                Delay::new(Duration::from_secs(8)).await;
                // do stuff...
                Outcome::passed()
            })
        }),
        Test::test("case2", || {
            task::spawn(async {
                Delay::new(Duration::from_secs(4)).await;
                // do stuff...
                Outcome::failed().error_message("foo")
            })
        }),
        Test::test("case3", || {
            task::spawn(async move {
                Delay::new(Duration::from_secs(6)).await;
                // do stuff ...
                Outcome::passed()
            })
        }),
    ];

    mimicaw::run_tests(&args, tests, |_, test_fn: TestFn| test_fn())
        .await
        .exit()
}
