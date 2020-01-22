use mimicaw::{Outcome, Test};
use std::{future::Future, pin::Pin};
use tokio::task;

#[tokio::main]
async fn main() {
    let tests = vec![
        Test::test("case1").context(Box::pin(async {
            task::spawn(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(10)).await;
                // do stuff...
                Outcome::Passed
            })
            .await
            .unwrap()
        })
            as Pin<Box<dyn Future<Output = Outcome> + 'static>>),
        Test::test("case2").context(Box::pin(async {
            task::spawn(async {
                tokio::time::delay_for(tokio::time::Duration::from_secs(6)).await;
                // do stuff...
                Outcome::Failed {
                    msg: Some("foo".into()),
                }
            })
            .await
            .unwrap()
        })),
        Test::test("case3_a_should_be_zero")
            .ignore(true)
            .context(Box::pin(async {
                task::spawn(async move {
                    tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;
                    // do stuff ...
                    Outcome::Passed
                })
                .await
                .unwrap()
            })),
    ];

    let status = mimicaw::run_tests(tests, std::convert::identity).await;
    std::process::exit(status);
}
