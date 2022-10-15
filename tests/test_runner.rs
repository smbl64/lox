use std::{path::Path, process::Output};

use assert_cmd::Command;
use walkdir::WalkDir;

// TODO: expect runtime error:

#[test]
fn run_all_files() {
    let dir = "./tests/data/";

    let entries = WalkDir::new(dir)
        .into_iter()
        .filter_map(|o| o.ok())
        .filter(|e| e.file_type().is_file());

    for entry in entries {
        let filename = entry.path();
        if is_blacklisted(filename.to_str().unwrap()) {
            continue;
        }

        print!("{} ... ", filename.display());

        let expect = find_expects(filename);
        let expected = expect.join("\n");

        let output = run_file(filename);
        //assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).unwrap();
        let stdout = stdout.trim_end();

        let stderr = String::from_utf8(output.stderr).unwrap();
        let stderr = stderr.trim_end();

        assert_eq!(expected, stdout, "stdout={}, stderr={}", stdout, stderr);

        println!("OK");
    }
}

fn run_file(filename: &Path) -> Output {
    let mut cmd = Command::cargo_bin("lox").unwrap();
    cmd.arg(filename).output().unwrap()
}

fn is_blacklisted(filename: &str) -> bool {
    let blacklist = vec![
        "394.lox",
        "benchmark/",
        "class", // Without slash
        "constructor/",
        "inheritance/",
        "method", // Without slash
        "super/",
        "this/",
    ];

    for s in blacklist {
        if filename.contains(s) {
            return true;
        }
    }

    false
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

//#[test]
//fn woot() {
//    let source = r#"
//        {
//            var b = b;
//        }
//    "#;

//    //let mut scanner = Scanner::new(source);
//    //let tokens = scanner.scan_tokens();

//    //let mut parser = Parser::new(tokens);
//    //let statements = parser.parse();
//    //assert!(statements.is_some());

//    //let statements = statements.unwrap();
//    //let mut interpreter = Interpreter::new();
//    //let mut resolver = Resolver::new(&mut interpreter);
//    //let res = resolver.resolve(&statements);
//    //assert!(res.is_ok());

//    //interpreter.interpret(&statements)

//    let mut lox = Lox::new();
//    let result = lox.run(source);
//    assert!(result.is_err());
//}
