use futures::prelude::*;
use mimicaw::{Outcome, Test};
use tokio::task;

#[tokio::main]
async fn main() {
    let tests = vec![
        Test::test("case1", {
            async {
                task::spawn(async {
                    tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
                    // do stuff...
                    Outcome::passed()
                })
                .await
                .unwrap()
            }
            .boxed_local()
        }),
        Test::test("case2", {
            async {
                task::spawn(async {
                    tokio::time::delay_for(tokio::time::Duration::from_secs(6)).await;
                    // do stuff...
                    Outcome::failed().error_message("foo")
                })
                .await
                .unwrap()
            }
            .boxed_local()
        }),
        Test::test("case3_a_should_be_zero", {
            async {
                task::spawn(async move {
                    tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;
                    // do stuff ...
                    Outcome::passed()
                })
                .await
                .unwrap()
            }
            .boxed_local()
        })
        .ignore(true),
    ];

    let status = mimicaw::run_tests(tests, |_, fut| fut).await;
    std::process::exit(status);
}
