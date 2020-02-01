use futures::executor::block_on;
use mimicaw::{Args, Outcome, Report, Test};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};
use sxd_document::{writer::Writer, Package};

fn main() -> io::Result<()> {
    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let tests = vec![
        Test::test("case1", "foo"),
        Test::test("case2", "bar"),
        Test::test("case3_long_computation", "baz").ignore(true),
        Test::test("case4", "The quick brown fox jumps over the lazy dog."),
    ];

    let report = block_on(mimicaw::run_tests_with_report(
        &args,
        tests,
        |_desc, data| async move {
            match data {
                "foo" | "baz" => Outcome::passed(),
                "bar" => Outcome::failed().error_message("`bar' is forbidden"),
                data => Outcome::failed().error_message(format!("unknown data: {}", data)),
            }
        },
    ))
    .unwrap_or_else(|st| st.exit());

    report_to_xml(&report, "report.xml")?;

    report.status().exit()
}

fn report_to_xml(report: &Report, path: impl AsRef<Path>) -> io::Result<()> {
    let package = Package::new();
    let doc = package.as_document();

    doc.root().append_child({
        let testsuites = doc.create_element("testsuites");
        testsuites.append_child({
            let testsuite = doc.create_element("testsuite");
            testsuite.set_attribute_value("name", "mimicaw xunit sample");
            testsuite.set_attribute_value("timestamp", &chrono::Utc::now().to_rfc3339());
            testsuite.append_child({
                let properties = doc.create_element("properties");
                properties.append_child({
                    let property = doc.create_element("property");
                    property.set_attribute_value("name", "module");
                    property.set_attribute_value("value", module_path!());
                    property
                });
                properties
            });

            for desc in &report.passed {
                let testcase = doc.create_element("testcase");
                testcase.set_attribute_value("name", desc.name());
                testcase.append_child(doc.create_element("passed"));
                testsuite.append_child(testcase);
            }

            for (desc, msg) in &report.failed {
                let testcase = doc.create_element("testcase");
                testcase.set_attribute_value("name", desc.name());
                testcase.append_child({
                    let failure = doc.create_element("failure");
                    if let Some(msg) = msg {
                        failure.set_attribute_value("message", &*msg);
                    }
                    failure
                });
                testsuite.append_child(testcase);
            }

            for (desc, reason) in report.skipped() {
                let testcase = doc.create_element("testcase");
                testcase.set_attribute_value("name", desc.name());
                testcase.append_child({
                    let skipped = doc.create_element("skipped");
                    skipped.set_attribute_value("message", reason);
                    skipped
                });
                testsuite.append_child(testcase);
            }

            testsuite
        });

        testsuites
    });

    let writer = Writer::new().set_single_quotes(false);

    let mut file = File::create(path)?;
    writer.format_document(&doc, &mut file)?;
    writeln!(file)?;
    file.flush()?;

    Ok(())
}
