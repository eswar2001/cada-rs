// src/granular.rs
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use serde_json::json;
use syn::spanned::Spanned;
use syn::ItemFn;

use crate::ast_parser::{extract_file_ast, extract_function_calls, extract_literals, format_node, get_source_location, remove_duplicates};
use crate::git_ops::{checkout_branch, checkout_commit};
use crate::types::{CalledFunctionChanges, FileASTData, SourceLocation, TypedLiteral};

// Get granular changes for functions between commits
pub fn get_granular_change_for_functions(rust_files: &[String], local_repo_path: &str, output_path: &str) {
    // Map to store file => function => changes
    let mut granular_changes = HashMap::new();
    
    // Get arguments from command line (similar to Go version)
    let args: Vec<String> = env::args().collect();
    let branch_name = &args[3];
    let current_commit = &args[4];
    
    println!("Using previous commit: {}", branch_name);
    println!("Current commit: {}", current_commit);
    
    // Step 1: Checkout the previous commit and extract all ASTs
    if let Err(e) = checkout_branch(branch_name, local_repo_path) {
        println!("Error checking out previous commit {}: {}", branch_name, e);
        return;
    }
    println!("Checked out previous commit: {}", branch_name);
    
    // Map to store all ASTs from previous commit
    let mut old_asts = HashMap::new();
    
    // Process all files in the previous commit
    for rust_file in rust_files {
        let full_path = Path::new(local_repo_path).join(rust_file);
        println!("Processing old AST for: {}", full_path.display());
        
        match extract_file_ast(full_path.to_str().unwrap_or("")) {
            Ok(ast) => {
                old_asts.insert(rust_file.clone(), ast);
            },
            Err(e) => {
                println!("Error extracting old AST for {}: {} (file might not exist in old commit)", rust_file, e);
                // Create an empty AST for files that don't exist in the old commit
                old_asts.insert(rust_file.clone(), FileASTData::empty(rust_file.clone()));
            }
        }
    }
    
    // Step 2: Checkout the current commit and extract all ASTs
    if let Err(e) = checkout_commit(current_commit, local_repo_path) {
        println!("Error checking out current commit {}: {}", current_commit, e);
        return;
    }
    println!("Checked out current commit: {}", current_commit);
    
    // Map to store all ASTs from current commit
    let mut new_asts = HashMap::new();
    
    // Process all files in the current commit
    for rust_file in rust_files {
        let full_path = Path::new(local_repo_path).join(rust_file);
        println!("Processing new AST for: {}", full_path.display());
        
        match extract_file_ast(full_path.to_str().unwrap_or("")) {
            Ok(ast) => {
                new_asts.insert(rust_file.clone(), ast);
            },
            Err(e) => {
                println!("Error extracting new AST for {}: {} (file might not exist in new commit)", rust_file, e);
                // Create an empty AST for files that don't exist in the new commit
                new_asts.insert(rust_file.clone(), FileASTData::empty(rust_file.clone()));
            }
        }
    }
    
    // Step 3: Analyze functions and methods for all files
    for rust_file in rust_files {
        let old_ast = old_asts.get(rust_file);
        let new_ast = new_asts.get(rust_file);
        
        // Skip files that don't exist in either commit
        if old_ast.is_none() && new_ast.is_none() {
            continue;
        }
        
        // Handle cases where the file exists only in one commit
        let old_ast = old_ast.cloned().unwrap_or_else(|| FileASTData::empty(rust_file.clone()));
        let new_ast = new_ast.cloned().unwrap_or_else(|| FileASTData::empty(rust_file.clone()));
        
        let mut file_changes = HashMap::new();
        
        // Check regular functions
        for (name, old_func) in &old_ast.functions {
            if let Some(new_func) = new_ast.functions.get(name) {
                // Function exists in both commits, compare them
                let old_code = format_node(old_func);
                let new_code = format_node(new_func);
                
                if old_code != new_code {
                    // Function has changed, analyze in detail
                    let changes = compare_called_functions(old_func, new_func, &old_ast, &new_ast);
                    file_changes.insert(name.clone(), changes);
                    println!("Added modified function: {}", name);
                }
            }
        }
        
        // Check methods too
        for (name, (_, old_method)) in &old_ast.methods {
            if let Some((_, new_method)) = new_ast.methods.get(name) {
                // Method exists in both commits, compare them
                let old_code = format_node(old_method);
                let new_code = format_node(new_method);
                
                if old_code != new_code {
                    // Method has changed, analyze in detail
                    let changes = compare_called_functions(old_method, new_method, &old_ast, &new_ast);
                    file_changes.insert(name.clone(), changes);
                    println!("Added modified method: {}", name);
                }
            }
        }
        
        if !file_changes.is_empty() {
            println!("Added {} changes for file {}", file_changes.len(), rust_file);
            granular_changes.insert(rust_file.clone(), file_changes);
        }
    }
    
    // Write the results to a JSON file
    if !granular_changes.is_empty() {
        println!("Found granular changes in {} files", granular_changes.len());
        
        match serde_json::to_string_pretty(&granular_changes) {
            Ok(granular_json) => {
                let granular_path = Path::new(output_path).join("function_changes_granular.json");
                if let Err(e) = fs::write(&granular_path, granular_json) {
                    println!("Error writing granular changes file: {}", e);
                } else {
                    println!("Wrote granular function changes to {}", granular_path.display());
                }
            },
            Err(e) => {
                println!("Error marshaling granular changes: {}", e);
            }
        }
    } else {
        println!("No granular changes found in any files");
        
        // Write an empty JSON object to the file
        let empty_json = "{}";
        let granular_path = Path::new(output_path).join("function_changes_granular.json");
        if let Err(e) = fs::write(&granular_path, empty_json) {
            println!("Error writing empty granular changes file: {}", e);
        } else {
            println!("Wrote empty granular changes file to {}", granular_path.display());
        }
    }
}

// Compare called functions between two function declarations
fn compare_called_functions(
    old_func: &ItemFn,
    new_func: &ItemFn,
    old_ast: &FileASTData,
    new_ast: &FileASTData,
) -> CalledFunctionChanges {
    let old_calls = extract_function_calls(old_func);
    let new_calls = extract_function_calls(new_func);
    
    // Find added and removed function calls
    let mut added_functions = Vec::new();
    for call in &new_calls {
        if !old_calls.contains(call) {
            added_functions.push(call.clone());
        }
    }
    
    let mut removed_functions = Vec::new();
    for call in &old_calls {
        if !new_calls.contains(call) {
            removed_functions.push(call.clone());
        }
    }
    
    // Extract literals
    let old_literals = extract_literals(old_func);
    let new_literals = extract_literals(new_func);
    
    // Find added and removed literals
    let mut added_literals = Vec::new();
    for lit in &new_literals {
        if !old_literals.iter().any(|old_lit| old_lit.type_name == lit.type_name && old_lit.value == lit.value) {
            added_literals.push(lit.clone());
        }
    }
    
    let mut removed_literals = Vec::new();
    for lit in &old_literals {
        if !new_literals.iter().any(|new_lit| new_lit.type_name == lit.type_name && new_lit.value == lit.value) {
            removed_literals.push(lit.clone());
        }
    }
    
    // Get source locations
    let old_function_src_loc = get_source_location(old_func.span(), &old_ast.file_path);
    let new_function_src_loc = get_source_location(new_func.span(), &new_ast.file_path);
    
    // Create the result
    let result = CalledFunctionChanges {
        added_functions: remove_duplicates(added_functions),
        removed_functions: remove_duplicates(removed_functions),
        added_literals,
        removed_literals,
        old_function_src_loc,
        new_function_src_loc,
    };
    
    // Log the changes found for debugging
    if !result.added_functions.is_empty() || !result.removed_functions.is_empty() ||
       !result.added_literals.is_empty() || !result.removed_literals.is_empty() {
        println!(
            "  - Function changes: +{} calls, -{} calls, +{} literals, -{} literals",
            result.added_functions.len(),
            result.removed_functions.len(),
            result.added_literals.len(),
            result.removed_literals.len()
        );
    }
    
    result
}