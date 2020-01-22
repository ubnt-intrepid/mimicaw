use futures::executor::block_on;
use mimicaw::{Outcome, Test};

fn main() {
    let tests = vec![
        Test::test("case1", "foo"),
        Test::test("case2", "bar"),
        Test::test("case3_long_computation", "baz"),
        Test::test("case4", "The quick brown fox jumps over the lazy dog."),
    ];

    let status = block_on(mimicaw::run_tests(tests, |_desc, data| {
        futures::future::ready(match data {
            "foo" | "baz" => Outcome::passed(),
            "bar" => Outcome::failed().error_message("`bar' is forbidden"),
            data => Outcome::failed().error_message(format!("unknown data: {}", data)),
        })
    }));
    std::process::exit(status);
}
