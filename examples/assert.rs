use futures::{executor::block_on, prelude::*};
use maybe_unwind::FutureMaybeUnwindExt as _;
use mimicaw::{Args, Outcome, Test};
use std::pin::Pin;

fn main() {
    maybe_unwind::set_hook();

    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let name = "Alice";
    let age = 14;
    let gender = "woman";

    let tests = vec![
        Test::test(
            "check_name",
            async {
                assert_eq!(name, "Alice");
            }
            .boxed_local(),
        ),
        Test::test(
            "check_age",
            async {
                assert_eq!(age, 14);
            }
            .boxed_local(),
        ),
        Test::test(
            "check_gender",
            async {
                assert_eq!(gender, "man");
            }
            .boxed_local(),
        ),
    ];

    block_on(mimicaw::run_tests(
        &args,
        tests,
        |_desc, fut: Pin<Box<dyn Future<Output = ()>>>| {
            async move {
                match std::panic::AssertUnwindSafe(fut).maybe_unwind().await {
                    Ok(()) => Outcome::passed(),
                    Err(unwind) => {
                        let location = match (unwind.file(), unwind.line(), unwind.column()) {
                            (Some(file), Some(line), Some(column)) => {
                                format!("{}:{}:{}", file, line, column)
                            }
                            (Some(file), _, _) => format!("{}:<unknown>", file),
                            _ => "<unknown>".into(),
                        };
                        Outcome::failed().error_message(format!(
                            "[{}] {}",
                            location,
                            unwind.payload_str()
                        ))
                    }
                }
            }
        },
    ))
    .exit()
}
