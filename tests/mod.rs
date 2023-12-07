use std::fs::create_dir_all;
use std::path::Path;

use walkdir::WalkDir;

use complex_code_spotter::{Complexity, OutputFormat, SnippetsProducer};

const SOURCE_PATH: &str = "data/seahorse/src";
const SNAPSHOT_PATH: &str = "tests/snapshots";
const TMP_DIR: &str = "complex-code-spotter";

#[test]
fn test_low_values() {
    // Snapshot path
    let snapshot_path = Path::new(SNAPSHOT_PATH).join("low_values");

    // Create low values directory
    create_dir_all(&snapshot_path).unwrap();

    // Temporary path
    let tmp_path = std::env::temp_dir().join(TMP_DIR);

    // Create directory in tmp directory
    create_dir_all(&tmp_path).unwrap();

    // Produce snippets
    SnippetsProducer::new()
        .complexities(vec![
            (Complexity::Cyclomatic, 1),
            (Complexity::Cognitive, 1),
        ])
        .output_format(OutputFormat::Json)
        .run(Path::new(SOURCE_PATH), tmp_path)
        .unwrap();

    // Iterate over temporary results
    for entry in WalkDir::new(tmp_path.join(OutputFormat::Json.into())).into_iter() {
        let entry = entry.unwrap();

        // Read snapshot

        insta::with_settings!({
            snapshot_path => snapshot_path.join(entry.path()),
            prepend_module_to_snapshot => false
        },{
            insta::assert_json_snapshot!(name, content);
        });
    }
}
