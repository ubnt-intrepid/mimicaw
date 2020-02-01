use futures::{executor::block_on, future::Future};
use maybe_unwind::FutureMaybeUnwindExt;
use mimicaw::{Args, Outcome, Test};
use std::{panic::UnwindSafe, pin::Pin};

pub use mimicaw_framework_macros::test;

pub type TestCase = Pin<Box<dyn Future<Output = ()> + UnwindSafe>>;

pub fn test_runner(tests: &[&dyn Fn() -> Test<TestCase>]) {
    maybe_unwind::set_hook();

    let args = Args::from_env().unwrap_or_else(|e| e.exit());
    let tests = tests.iter().map(|factory| (*factory)());
    let status = block_on(mimicaw::run_tests(
        &args,
        tests,
        |_desc, test: TestCase| async move {
            match test.maybe_unwind().await {
                Ok(()) => Outcome::passed(),
                Err(unwind) => Outcome::failed().error_message(unwind.to_string()),
            }
        },
    ));

    status.exit();
}
