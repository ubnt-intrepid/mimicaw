use mimicaw::Test;
use tokio::task;

#[tokio::main]
async fn main() {
    let tests = vec![
        Test::test("case1", async {
            let handle = task::spawn(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
                // do stuff...
                Ok(())
            });
            handle.await.unwrap()
        }),
        Test::test("case2", async {
            let handle = task::spawn(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(6)).await;
                // do stuff...
                Err(Some("foo".into()))
            });
            handle.await.unwrap()
        }),
        Test::test("case3_a_should_be_zero", async {
            let handle = task::spawn(async move {
                tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;
                // do stuff ...
                Ok(())
            });
            handle.await.unwrap()
        })
        .ignored(true),
    ];

    let status = mimicaw::run_tests(tests).await;
    std::process::exit(status);
}
