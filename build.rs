use std::env;
use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

static TEST_DATA: &'static str = "./tests/data/";
static TEST_TEMPLATE: &'static str = r#"
    #[test]
    fn {test_name}() {
        let filename = Path::new("{filename}");
        do_test(filename);
    }
"#;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("test_files.rs");

    let file = fs::File::create(&dest_path).unwrap();
    let mut buf = BufWriter::new(file);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", TEST_DATA);

    for entry in get_all_files() {
        let filename = entry.path().to_str().unwrap();
        if should_skip(filename) {
            continue;
        }

        let test_name = filename
            .replace("./", "")
            .replace("/", "_")
            .replace(".lox", "")
            .replace("tests_data_", "");

        let test_case = TEST_TEMPLATE
            .replace("{test_name}", &test_name)
            .replace("{filename}", filename);

        write!(&mut buf, "{}", test_case).unwrap();
    }
}

fn get_all_files() -> Vec<walkdir::DirEntry> {
    WalkDir::new(TEST_DATA)
        .into_iter()
        .filter_map(|o| o.ok())
        .filter(|e| e.file_type().is_file())
        .collect()
}

fn should_skip(filename: &str) -> bool {
    let skip_list = vec!["benchmark/"];

    for s in skip_list {
        if filename.contains(s) {
            return true;
        }
    }

    false
}
