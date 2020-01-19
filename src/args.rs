use getopts::Options;
use std::{
    convert::{TryFrom, TryInto},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) list: bool,
    pub(crate) filter: Option<String>,
    pub(crate) filter_exact: bool,
    pub(crate) run_ignored: bool,
    pub(crate) run_tests: bool,
    pub(crate) run_benchmarks: bool,
    pub(crate) logfile: Option<PathBuf>,
    pub(crate) nocapture: bool,
    pub(crate) color: ColorConfig,
    pub(crate) format: OutputFormat,
    pub(crate) test_threads: Option<usize>,
    pub(crate) skip: Vec<String>,
}

impl Args {
    pub(crate) fn from_env() -> Result<Self, i32> {
        let args: Vec<_> = std::env::args().collect();
        let parser = Parser::new();
        match parser.parse_args(&args) {
            Ok(args) => args.ok_or(0),
            Err(err) => {
                eprintln!("error: {}", err);
                Err(crate::ERROR_STATUS_CODE)
            }
        }
    }
}

struct TestThreads(usize);

impl TryFrom<String> for TestThreads {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let n = s.parse().map_err(|e| {
            format!(
                "argument for --test-threads must be a number > 0 (error: {})",
                e
            )
        })?;
        if n == 0 {
            return Err("argument for --test-threads must not be 0".into());
        }
        Ok(Self(n))
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ColorConfig {
    Auto,
    Always,
    Never,
}

impl TryFrom<String> for ColorConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "auto" => Ok(ColorConfig::Auto),
            "always" => Ok(ColorConfig::Always),
            "never" => Ok(ColorConfig::Never),
            v => Err(format!(
                "argument for --color must be auto, always, or never (was {})",
                v
            )
            .into()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    Pretty,
    Terse,
    Json,
}

impl TryFrom<String> for OutputFormat {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "pretty" => Ok(OutputFormat::Pretty),
            "terse" => Ok(OutputFormat::Terse),
            "json" => Ok(OutputFormat::Json),
            s => Err(format!(
                "argument for --format must be pretty, terse, or json (was {})",
                s
            )
            .into()),
        }
    }
}

struct Parser(Options);

impl Parser {
    fn new() -> Self {
        let mut opts = Options::new();
        opts.optflag("", "ignored", "Run only ignored tests");
        opts.optflag("", "test", "Run tests and not benchmarks");
        opts.optflag("", "bench", "Run benchmarks instead of tests");
        opts.optflag("", "list", "List all tests and benchmarks");
        opts.optflag("h", "help", "Display this message (longer with --help)");
        opts.optopt(
            "",
            "logfile",
            "Write logs to the specified file instead \
             of stdout",
            "PATH",
        );
        opts.optflag(
            "",
            "nocapture",
            "don't capture stdout/stderr of each \
             task, allow printing directly",
        );
        opts.optopt(
            "",
            "test-threads",
            "Number of threads used for running tests \
             in parallel",
            "n_threads",
        );
        opts.optmulti(
            "",
            "skip",
            "Skip tests whose names contain FILTER (this flag can \
             be used multiple times)",
            "FILTER",
        );
        opts.optflag(
            "q",
            "quiet",
            "Display one character per test instead of one line. \
             Alias to --format=terse",
        );
        opts.optflag(
            "",
            "exact",
            "Exactly match filters rather than by substring",
        );
        opts.optopt(
            "",
            "color",
            "Configure coloring of output:
                auto   = colorize if stdout is a tty and tests are run on serially (default);
                always = always colorize output;
                never  = never colorize output;",
            "auto|always|never",
        );
        opts.optopt(
            "",
            "format",
            "Configure formatting of output:
                pretty = Print verbose output;
                terse  = Display one character per test;
                json   = Output a json document",
            "pretty|terse|json",
        );

        Self(opts)
    }

    fn print_usage(&self, binary: &str) {
        let progname = Path::new(binary)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(binary);

        let message = format!("Usage: {} [OPTIONS] [FILTER]", progname);
        println!(
            r#"{usage}
    
    The FILTER string is tested against the name of all tests, and only those
    tests whose names contain the filter are run.
    
    By default, all tests are run in parallel. This can be altered with the
    --test-threads flag or the RUST_TEST_THREADS environment variable when running
    tests (set it to 1).
    
    All tests have their standard output and standard error captured by default.
    This can be overridden with the --nocapture flag or setting RUST_TEST_NOCAPTURE
    environment variable to a value other than "0". Logging is not captured by default."#,
            usage = self.0.usage(&message)
        );
    }

    fn parse_args(&self, args: &[String]) -> Result<Option<Args>, Box<dyn std::error::Error>> {
        let matches = self.0.parse(args.get(1..).unwrap_or(args))?;
        if matches.opt_present("h") {
            self.print_usage(&args[0]);
            return Ok(None);
        }

        let filter = matches.free.get(0).cloned();
        let run_ignored = matches.opt_present("ignored");
        let quiet = matches.opt_present("quiet");
        let filter_exact = matches.opt_present("exact");
        let list = matches.opt_present("list");
        let logfile = matches.opt_str("logfile").map(|s| PathBuf::from(&s));

        let run_benchmarks = matches.opt_present("bench");
        let run_tests = !run_benchmarks || matches.opt_present("test");

        let nocapture = matches.opt_present("nocapture") || {
            std::env::var("RUST_TEST_NOCAPTURE")
                .ok()
                .map_or(false, |val| &val != "0")
        };

        let test_threads = match matches.opt_str("test-threads") {
            Some(n_str) => n_str.try_into().map(|TestThreads(n)| Some(n))?,
            None => None,
        };

        let color = match matches.opt_str("color") {
            Some(s) => s.try_into()?,
            None => ColorConfig::Auto,
        };

        let format = match matches.opt_str("format") {
            Some(s) => s.try_into()?,
            None if quiet => OutputFormat::Terse,
            None => OutputFormat::Pretty,
        };

        let skip = matches.opt_strs("skip");

        Ok(Some(Args {
            list,
            filter,
            filter_exact,
            run_ignored,
            run_tests,
            run_benchmarks,
            logfile,
            nocapture,
            color,
            format,
            test_threads,
            skip,
        }))
    }
}
