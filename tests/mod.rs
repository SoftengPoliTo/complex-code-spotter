use std::path::Path;

use complex_code_spotter::{Complexity, OutputFormat, SnippetsProducer};

const SOURCE_PATH: &str = "data/seahorse/src";
const SNAPSHOT_PATH: &str = "snapshots";
const TMP_DIR: &str = "complex-code-spotter";

fn run_tests(subdir: &str, complexities: Vec<(Complexity, usize)>) {
    // Snapshot path
    let snapshot_path = Path::new(SNAPSHOT_PATH).join(subdir);

    // Temporary path
    let tmp_path = std::env::temp_dir().join(TMP_DIR);

    // Retrieve snippets
    let snippets = SnippetsProducer::new()
        .complexities(complexities)
        .output_format(OutputFormat::Json)
        .run(Path::new(SOURCE_PATH), &tmp_path)
        .unwrap();

    if let Some(snippets) = snippets {
        // Iterate over temporary results
        for snippet in snippets {
            // Snapshot name
            let name = snippet
                .source_path
                .as_path()
                .to_str()
                .unwrap()
                .replace("/", "_");

            insta::with_settings!({
                snapshot_path => &snapshot_path,
                // Sort hashmaps to avoid having different orders in snapshots
                sort_maps => true,
                prepend_module_to_snapshot => false
            },{
                insta::assert_yaml_snapshot!(name, snippet);
            });
        }
    } else {
        assert!(true);
    }
}

#[test]
fn seahorse_high_thresholds() {
    run_tests(
        "high_thresholds",
        vec![(Complexity::Cyclomatic, 15), (Complexity::Cognitive, 15)],
    );
}

#[test]
fn seahorse_low_thresholds() {
    run_tests(
        "low_thresholds",
        vec![(Complexity::Cyclomatic, 1), (Complexity::Cognitive, 1)],
    );
}
