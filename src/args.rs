#![allow(missing_docs)]

use crate::ExitStatus;
use getopts::Options;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

/// Command line arguments.
#[derive(Debug)]
#[non_exhaustive]
pub struct Args {
    pub list: bool,
    pub filter: Option<String>,
    pub filter_exact: bool,
    pub run_ignored: bool,
    pub run_tests: bool,
    pub run_benchmarks: bool,
    pub logfile: Option<PathBuf>,
    pub nocapture: bool,
    pub color: ColorConfig,
    pub format: OutputFormat,
    pub test_threads: Option<usize>,
    pub skip: Vec<String>,
}

impl Args {
    /// Parse command line arguments.
    pub fn from_env() -> Result<Self, ExitStatus> {
        let parser = Parser::new();
        match parser.parse_args() {
            Ok(Some(args)) => Ok(args),
            Ok(None) => {
                parser.print_usage();
                Err(ExitStatus::OK)
            }
            Err(err) => {
                eprintln!("CLI argument error: {}", err);
                Err(ExitStatus::FAILED)
            }
        }
    }

    pub(crate) fn is_filtered(&self, name: &str) -> bool {
        if let Some(ref filter) = self.filter {
            if self.filter_exact && name != filter {
                return true;
            }
            if !name.contains(filter) {
                return true;
            }
        }

        for skip_filter in &self.skip {
            if self.filter_exact && name != skip_filter {
                return true;
            }
            if !name.contains(skip_filter) {
                return true;
            }
        }

        false
    }
}

struct TestThreads(usize);

impl FromStr for TestThreads {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

/// The color configuration.
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ColorConfig {
    Auto,
    Always,
    Never,
}

impl FromStr for ColorConfig {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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

/// The output format.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutputFormat {
    Pretty,
    Terse,
    Json,
}

impl FromStr for OutputFormat {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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

struct Parser {
    args: Vec<String>,
    opts: Options,
}

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
            "Write logs to the specified file instead of stdout.
             (placeholder. not implemented yet)",
            "PATH",
        );
        opts.optflag(
            "",
            "nocapture",
            "don't capture stdout/stderr of each task, allow printing directly.
             (placeholder, not implemented yet)",
        );
        opts.optopt(
            "",
            "test-threads",
            "Number of threads used for running tests in parallel.
             (placeholder, not implemented yet)",
            "n_threads",
        );
        opts.optmulti(
            "",
            "skip",
            "Skip tests whose names contain FILTER (this flag can be used multiple times)",
            "FILTER",
        );
        opts.optflag(
            "q",
            "quiet",
            "Display one character per test instead of one line. Alias to --format=terse",
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
                json   = Output a json document (placeholder, not implemented yet)",
            "pretty|terse|json",
        );

        Self {
            args: std::env::args().collect(),
            opts,
        }
    }

    fn print_usage(&self) {
        let binary = &self.args[0];
        let progname = Path::new(binary)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(binary);

        let message = format!("Usage: {} [OPTIONS] [FILTER]", progname);
        eprintln!(
            r#"{usage}
    
    The FILTER string is tested against the name of all tests, and only those
    tests whose names contain the filter are run."#,
            usage = self.opts.usage(&message)
        );
    }

    fn parse_args(&self) -> Result<Option<Args>, Box<dyn std::error::Error>> {
        let args = &self.args[..];

        let matches = self.opts.parse(args.get(1..).unwrap_or(args))?;
        if matches.opt_present("h") {
            return Ok(None);
        }

        let filter = matches.free.get(0).cloned();
        let run_ignored = matches.opt_present("ignored");
        let quiet = matches.opt_present("quiet");
        let filter_exact = matches.opt_present("exact");
        let list = matches.opt_present("list");
        let logfile = matches.opt_get("logfile")?;

        let run_benchmarks = matches.opt_present("bench");
        let run_tests = !run_benchmarks || matches.opt_present("test");

        let nocapture = matches.opt_present("nocapture") || {
            std::env::var("RUST_TEST_NOCAPTURE")
                .ok()
                .map_or(false, |val| &val != "0")
        };

        let test_threads = matches.opt_get("test-threads")?.map(|TestThreads(n)| n);

        let color = matches.opt_get("color")?.unwrap_or(ColorConfig::Auto);

        let format = matches.opt_get("format")?.unwrap_or_else(|| {
            if quiet {
                OutputFormat::Terse
            } else {
                OutputFormat::Pretty
            }
        });

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
