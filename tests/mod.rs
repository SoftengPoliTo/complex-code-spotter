use std::fs::{create_dir_all, File};
use std::io::BufReader;
use std::path::Path;

use walkdir::WalkDir;

use complex_code_spotter::{Complexity, OutputFormat, Snippets, SnippetsProducer};

const SOURCE_PATH: &str = "data/seahorse/src";
const SNAPSHOT_PATH: &str = "tests/snapshots";
const TMP_DIR: &str = "complex-code-spotter";

#[inline(always)]
fn read_snippets(path: &Path) -> Snippets {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn run_tests(subdir: &str, complexities: Vec<(Complexity, usize)>) {
    // Snapshot path
    let snapshot_path = Path::new(SNAPSHOT_PATH).join(subdir);

    // Create low values directory
    create_dir_all(&snapshot_path).unwrap();

    // Temporary path
    let tmp_path = std::env::temp_dir().join(TMP_DIR);

    // Create directory in tmp directory
    create_dir_all(&tmp_path).unwrap();

    // Produce snippets
    SnippetsProducer::new()
        .complexities(complexities)
        .output_format(OutputFormat::Json)
        .run(Path::new(SOURCE_PATH), &tmp_path)
        .unwrap();

    // Iterate over temporary results
    for entry in WalkDir::new(tmp_path).into_iter() {
        let entry = entry.unwrap();
        let entry = entry.path();

        // Read file
        let snippet = read_snippets(entry);

        // Snapshot name
        let name = entry.file_name().and_then(|v| v.to_str()).unwrap();

        insta::with_settings!({
            snapshot_path => snapshot_path.join(entry),
            prepend_module_to_snapshot => false
        },{
            insta::assert_json_snapshot!(name, snippet);
        });
    }
}

#[test]
fn seahorse_high_thresholds() {
    run_tests(
        "high_values",
        vec![(Complexity::Cyclomatic, 15), (Complexity::Cognitive, 15)],
    );
}

#[test]
fn seahorse_low_thresholds() {
    run_tests(
        "low_values",
        vec![(Complexity::Cyclomatic, 1), (Complexity::Cognitive, 1)],
    );
}
