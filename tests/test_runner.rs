use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

include!(concat!(env!("OUT_DIR"), "/test_files.rs"));

// These functions are used by the included tests above
// See `build.rs` for the code that generates the tests.

fn do_test(filename: &Path) {
    let output = find_expects(filename, "expect: ", false, false);

    let mut errors = vec![];
    // Resolver errors
    errors.extend(find_expects(filename, "[line ", false, true));

    // Runtime errors
    errors.extend(find_expects(filename, "expect runtime error: ", true, false));

    // Parser or resolver errors that are put on a specific line (so no '[line X]
    // prefix')
    errors.extend(find_expects(filename, "Error at ", true, true));

    // Join them
    let expected = output.join("\n");
    let expected_error = errors.join("\n");

    let output = run_file(filename);

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = stdout.trim_end();

    let stderr = String::from_utf8(output.stderr).unwrap();
    let stderr = stderr.trim_end();

    assert_eq!(stdout, expected, "generated output != expected output");
    assert_eq!(stderr, expected_error, "generated error != expected error");
}

fn run_file(filename: &Path) -> Output {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg(filename).output().unwrap()
}

fn find_expects(
    filename: &Path,
    prefix: &str,
    prefix_line_nr: bool,
    include_prefix: bool,
) -> Vec<String> {
    let content = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("failed to read {}", filename.display()));

    let comment = "// ";
    let pattern = format!("{}{}", comment, prefix);

    let mut result = vec![];
    for (line_nr, line) in content.lines().enumerate() {
        // Find the prefix in current line
        let indices: Vec<_> = line.match_indices(&pattern).collect();
        if indices.is_empty() {
            continue;
        }

        let (idx, _) = indices.last().unwrap();
        let start_idx =
            if include_prefix { idx + comment.len() } else { idx + comment.len() + prefix.len() };

        let target = &line[start_idx..];

        if prefix_line_nr {
            result.push(format!("[line {}] {target}", line_nr + 1));
        } else {
            result.push(target.into());
        }
    }

    result
}
