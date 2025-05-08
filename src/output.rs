// src/output.rs
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::types::DetailedChanges;

// Create all the output JSON files
pub fn create_output_files(all_changes: &[DetailedChanges], output_path: &str) {
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(output_path) {
        println!("Error creating output directory: {}", e);
        return;
    }

    // Write detailed changes to a single file
    match serde_json::to_string_pretty(all_changes) {
        Ok(all_changes_json) => {
            let all_changes_path = Path::new(output_path).join("all_code_changes.json");
            if let Err(e) = fs::write(&all_changes_path, all_changes_json) {
                println!("Error writing all changes file: {}", e);
            }
        },
        Err(e) => {
            println!("Error marshaling all changes: {}", e);
        }
    }

    // Create separate files for each type of change
    create_type_specific_file(
        all_changes,
        "function_changes.json",
        |c| (&c.added_functions, &c.modified_functions, &c.deleted_functions),
        output_path,
    );

    create_type_specific_file(
        all_changes,
        "type_changes.json",
        |c| (&c.added_types, &c.modified_types, &c.deleted_types),
        output_path,
    );

    create_type_specific_file(
        all_changes,
        "interface_changes.json",
        |c| (&c.added_interfaces, &c.modified_interfaces, &c.deleted_interfaces),
        output_path,
    );

    create_type_specific_file(
        all_changes,
        "method_changes.json",
        |c| (&c.added_methods, &c.modified_methods, &c.deleted_methods),
        output_path,
    );
}

// Structure for type-specific changes
#[derive(Serialize, Deserialize)]
struct TypeSpecificChanges {
    added: Vec<HashMap<String, serde_json::Value>>,
    modified: Vec<HashMap<String, serde_json::Value>>,
    deleted: Vec<HashMap<String, serde_json::Value>>,
}

// Create a file for a specific type of change
fn create_type_specific_file(
    all_changes: &[DetailedChanges],
    filename: &str,
    extractor: impl Fn(&DetailedChanges) -> (&Vec<Vec<String>>, &Vec<Vec<String>>, &Vec<Vec<String>>),
    output_path: &str,
) {
    let mut changes = TypeSpecificChanges {
        added: Vec::new(),
        modified: Vec::new(),
        deleted: Vec::new(),
    };

    for c in all_changes {
        let (added, modified, deleted) = extractor(c);

        for item in added {
            let mut map = HashMap::new();
            map.insert("module".to_string(), json!(c.module_name));
            map.insert("name".to_string(), json!(item[0]));
            map.insert("code".to_string(), json!(item[1]));
            changes.added.push(map);
        }

        for item in modified {
            let mut map = HashMap::new();
            map.insert("module".to_string(), json!(c.module_name));
            map.insert("name".to_string(), json!(item[0]));
            map.insert("oldCode".to_string(), json!(item[1]));
            map.insert("newCode".to_string(), json!(item[2]));
            changes.modified.push(map);
        }

        for item in deleted {
            let mut map = HashMap::new();
            map.insert("module".to_string(), json!(c.module_name));
            map.insert("name".to_string(), json!(item[0]));
            map.insert("code".to_string(), json!(item[1]));
            changes.deleted.push(map);
        }
    }

    match serde_json::to_string_pretty(&changes) {
        Ok(changes_json) => {
            let file_path = Path::new(output_path).join(filename);
            if let Err(e) = fs::write(&file_path, changes_json) {
                println!("Error writing {}: {}", filename, e);
            }
        },
        Err(e) => {
            println!("Error marshaling {}: {}", filename, e);
        }
    }
}