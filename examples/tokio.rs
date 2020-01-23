use mimicaw::{Args, Outcome, Test};
use tokio::task;

type TestFn = fn() -> task::JoinHandle<Outcome>;

#[tokio::main]
async fn main() {
    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let tests: Vec<Test<TestFn>> = vec![
        Test::test("case1", || {
            task::spawn(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
                // do stuff...
                Outcome::passed()
            })
        }),
        Test::test("case2", || {
            task::spawn(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(6)).await;
                // do stuff...
                Outcome::failed().error_message("foo")
            })
        }),
        Test::<TestFn>::test("case3_a_should_be_zero", || {
            task::spawn(async move {
                tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;
                // do stuff ...
                Outcome::passed()
            })
        })
        .ignore(true),
    ];

    mimicaw::run_tests(&args, tests, |_, test_fn: TestFn| {
        let handle = test_fn();
        async move { handle.await.unwrap() }
    })
    .await
    .exit()
}
