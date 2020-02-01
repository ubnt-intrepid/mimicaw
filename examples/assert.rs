use futures::executor::block_on;
use maybe_unwind::maybe_unwind;
use mimicaw::{Args, Outcome, Test};
use std::panic::UnwindSafe;

fn main() {
    maybe_unwind::set_hook();

    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let name = "Alice";
    let age = 14;
    let gender = "woman";

    let tests: Vec<Test<Box<dyn Fn() + UnwindSafe>>> = vec![
        Test::test(
            "check_name",
            Box::new(|| {
                assert_eq!(name, "Alice");
            }),
        ),
        Test::test(
            "check_age",
            Box::new(|| {
                assert_eq!(age, 14);
            }),
        ),
        Test::test(
            "check_gender",
            Box::new(|| {
                assert_eq!(gender, "man");
            }),
        ),
    ];

    block_on(mimicaw::run_tests(
        &args,
        tests,
        |_desc, f: Box<dyn Fn() + UnwindSafe>| async move {
            match maybe_unwind(f) {
                Ok(()) => Outcome::passed(),
                Err(unwind) => {
                    let location = unwind
                        .location()
                        .map_or("<unknown>".into(), |loc| loc.to_string());
                    Outcome::failed().error_message(format!(
                        "[{}] {}",
                        location,
                        unwind.payload_str()
                    ))
                }
            }
        },
    ))
    .exit()
}
