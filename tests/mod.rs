use std::path::Path;

use walkdir::WalkDir;

const SNAPSHOT_PATH: &str = "../snapshots/low_values";

#[test]
fn test_low_values() {
    // Create temporary results

    // Iterate over temporary results
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Read snapshot

        insta::with_settings!({
            snapshot_path => Path::new(snapshot_path).join(entry),
            prepend_module_to_snapshot => false
        },{
            insta::assert_json_snapshot!(name, content);
        });
    }
}
