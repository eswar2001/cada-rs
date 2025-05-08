// src/main.rs
use std::env;
use std::process;

mod ast_parser;
mod differ;
mod git_ops;
mod granular;
mod output;
mod types;


fn main() {
    // Check arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("Usage: rust-ast-differ <repoUrl> <localRepoPath> <branchName> <currentCommit> [outputPath]");
        process::exit(1);
    }

    let repo_url = &args[1];
    let local_repo_path = &args[2];
    let branch_name = &args[3];
    let current_commit = &args[4];

    // Set default output path if not provided
    let output_path = if args.len() >= 6 {
        args[5].clone()
    } else {
        "./".to_string()
    };

    // Clone repository if it doesn't exist
    git_ops::clone_repo(repo_url, branch_name, local_repo_path);

    // Get changed files between commits
    let changed_files = match git_ops::get_changed_files(branch_name, local_repo_path) {
        Ok(files) => files,
        Err(e) => {
            println!("Error getting changed files: {}", e);
            process::exit(1);
        }
    };

    // Get lists of new and deleted files
    let new_files = match git_ops::get_new_files(branch_name, current_commit, local_repo_path) {
        Ok(files) => files,
        Err(e) => {
            println!("Warning: Error getting new files: {}", e);
            vec![]
        }
    };
    
    let deleted_files = match git_ops::get_deleted_files(branch_name, current_commit, local_repo_path) {
        Ok(files) => files,
        Err(e) => {
            println!("Warning: Error getting deleted files: {}", e);
            vec![]
        }
    };

    // Create maps for quick lookup
    let mut new_file_map = std::collections::HashMap::new();
    let mut deleted_file_map = std::collections::HashMap::new();

    for file in &new_files {
        new_file_map.insert(file.clone(), true);
        println!("New file detected: {}", file);
    }

    for file in &deleted_files {
        deleted_file_map.insert(file.clone(), true);
        println!("Deleted file detected: {}", file);
    }

    println!("Modified files: {:?}", changed_files);

    // Filter only Rust files
    let rust_files: Vec<String> = changed_files
        .iter()
        .filter(|file| file.ends_with(".rs"))
        .cloned()
        .collect();

    if rust_files.is_empty() {
        println!("No Rust files were modified between the specified commits");
        process::exit(0);
    }

    // First checkout the branch to ensure we're starting from the right point
    if let Err(e) = git_ops::checkout_branch(branch_name, local_repo_path) {
        println!("Error checking out branch {}: {}", branch_name, e);
        println!("Trying alternative checkout approaches...");
        
        // Try to checkout the commit directly
        if let Err(e) = git_ops::checkout_commit(&format!("{}^{{commit}}", branch_name), local_repo_path) {
            println!("Error checking out commit directly: {}", e);
            process::exit(1);
        }
        
        println!("Successfully checked out commit directly.");
    }

    // Process all Rust files to find changes
    let all_changes = differ::process_rust_files(
        &rust_files,
        local_repo_path,
        branch_name,
        current_commit,
        &new_file_map,
        &deleted_file_map,
    );

    // Create output files with the changes
    output::create_output_files(&all_changes, &output_path);

    // Get granular changes for functions
    granular::get_granular_change_for_functions(&rust_files, local_repo_path, &output_path);

    println!("AST diff complete. Check output files for details.");
}