use futures::executor::block_on;
use mimicaw::{Args, Outcome, Test};

fn main() {
    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let tests = vec![
        Test::test("case1", "foo"),
        Test::test("case2", "bar"),
        Test::test("case3_long_computation", "baz"),
        Test::test("case4", "The quick brown fox jumps over the lazy dog."),
    ];

    block_on(mimicaw::run_tests(&args, tests, |_desc, data| async move {
        match data {
            "foo" | "baz" => Outcome::passed(),
            "bar" => Outcome::failed().error_message("`bar' is forbidden"),
            data => Outcome::failed().error_message(format!("unknown data: {}", data)),
        }
    }))
    .exit()
}
