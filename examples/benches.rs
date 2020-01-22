use futures::executor::block_on;
use futures_timer::Delay;
use mimicaw::Test;
use std::time::Duration;

fn main() {
    let tests = vec![
        Test::bench("bench1", async {
            // do stuff...
            Delay::new(Duration::from_secs(4)).await;
            Ok((1274, 23))
        }),
        Test::bench("bench2", async {
            // do stuff...
            Delay::new(Duration::from_secs(8)).await;
            Ok((23, 430))
        })
        .ignored(true),
        Test::test("test1", async {
            // do stuff...
            Delay::new(Duration::from_secs(2)).await;
            Ok(())
        }),
        Test::test("test2", async {
            // do stuff...
            Delay::new(Duration::from_secs(6)).await;
            Ok(())
        })
        .ignored(true),
    ];

    let status = block_on(mimicaw::run_tests(tests));
    std::process::exit(status);
}
