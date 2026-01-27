use crate::executor::FailureDetail;
use flint_core::results::TestResult;
use std::time::Duration;

/// Print results as JSON to stdout
pub fn print_json(results: &[TestResult], failures: &[(String, FailureDetail)], elapsed: Duration) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let failed = total - passed;

    let failure_objects: Vec<serde_json::Value> = failures
        .iter()
        .map(|(name, detail)| {
            serde_json::json!({
                "test": name,
                "tick": detail.tick,
                "expected": detail.expected,
                "actual": detail.actual,
                "position": detail.position,
            })
        })
        .collect();

    let test_objects: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.test_name,
                "success": r.success,
                "total_ticks": r.total_ticks,
                "execution_time_ms": r.execution_time_ms,
            })
        })
        .collect();

    let output = serde_json::json!({
        "summary": {
            "total": total,
            "passed": passed,
            "failed": failed,
            "duration_secs": elapsed.as_secs_f64(),
        },
        "tests": test_objects,
        "failures": failure_objects,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// Print results in TAP (Test Anything Protocol) version 13 format
pub fn print_tap(results: &[TestResult], failures: &[(String, FailureDetail)]) {
    println!("TAP version 13");
    println!("1..{}", results.len());

    // Build a lookup from test name to failure detail
    let failure_map: std::collections::HashMap<&str, &FailureDetail> = failures
        .iter()
        .map(|(name, detail)| (name.as_str(), detail))
        .collect();

    for (i, result) in results.iter().enumerate() {
        let number = i + 1;
        if result.success {
            println!("ok {} - {}", number, result.test_name);
        } else {
            println!("not ok {} - {}", number, result.test_name);
            if let Some(detail) = failure_map.get(result.test_name.as_str()) {
                println!("  ---");
                println!(
                    "  message: \"expected {}, got {}\"",
                    detail.expected, detail.actual
                );
                println!(
                    "  at: [{}, {}, {}]",
                    detail.position[0], detail.position[1], detail.position[2]
                );
                println!("  tick: {}", detail.tick);
                println!("  ...");
            }
        }
    }
}

/// Print results in JUnit XML format
pub fn print_junit(
    results: &[TestResult],
    failures: &[(String, FailureDetail)],
    elapsed: Duration,
) {
    let total = results.len();
    let failed = results.iter().filter(|r| !r.success).count();

    let failure_map: std::collections::HashMap<&str, &FailureDetail> = failures
        .iter()
        .map(|(name, detail)| (name.as_str(), detail))
        .collect();

    println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    println!(
        r#"<testsuites tests="{}" failures="{}" time="{:.3}">"#,
        total,
        failed,
        elapsed.as_secs_f64()
    );
    println!(
        r#"  <testsuite name="flintmc" tests="{}" failures="{}" time="{:.3}">"#,
        total,
        failed,
        elapsed.as_secs_f64()
    );

    for result in results {
        // Split test name into classname (directory path) and name (leaf)
        let (classname, name) = match result.test_name.rsplit_once('/') {
            Some((prefix, leaf)) => (prefix, leaf),
            None => ("", result.test_name.as_str()),
        };

        let time = result.execution_time_ms as f64 / 1000.0;

        if result.success {
            println!(
                r#"    <testcase classname="{}" name="{}" time="{:.3}" />"#,
                xml_escape(classname),
                xml_escape(name),
                time
            );
        } else {
            println!(
                r#"    <testcase classname="{}" name="{}" time="{:.3}">"#,
                xml_escape(classname),
                xml_escape(name),
                time
            );
            if let Some(detail) = failure_map.get(result.test_name.as_str()) {
                println!(
                    r#"      <failure message="expected {}, got {} at ({},{},{}) tick {}"/>"#,
                    xml_escape(&detail.expected),
                    xml_escape(&detail.actual),
                    detail.position[0],
                    detail.position[1],
                    detail.position[2],
                    detail.tick
                );
            } else {
                println!(r#"      <failure message="assertion failed"/>"#);
            }
            println!("    </testcase>");
        }
    }

    println!("  </testsuite>");
    println!("</testsuites>");
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
