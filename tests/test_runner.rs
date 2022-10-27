use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

include!(concat!(env!("OUT_DIR"), "/test_files.rs"));

// These functions are used by the included tests above
// See `build.rs` for the code that generates the tests.

// TODO: expect runtime error:
fn do_test(filename: &Path) {
    let expect = find_expects(filename);
    let expected = expect.join("\n");

    let output = run_file(filename);
    //assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = stdout.trim_end();

    let stderr = String::from_utf8(output.stderr).unwrap();
    let stderr = stderr.trim_end();

    assert_eq!(expected, stdout, "Expected != stdout\n  stdout={stdout}\n  stderr={stderr}\n");
}

fn run_file(filename: &Path) -> Output {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg(filename).output().unwrap()
}

fn find_expects(filename: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("failed to read {}", filename.display()));

    let expect_str = "// expect: ";
    let mut result = vec![];
    for line in content.lines() {
        let mut indices: Vec<_> = line.match_indices(expect_str).collect();
        if indices.is_empty() {
            continue;
        }

        let (idx, _) = indices.pop().unwrap();
        let target = &line[idx + expect_str.len()..];
        result.push(target.into());
    }

    result
}
