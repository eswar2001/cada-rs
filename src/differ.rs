// src/differ.rs
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use syn::{Item, ItemFn, ItemTrait,ItemImpl};

use crate::ast_parser::{extract_file_ast, extract_module_name, format_node, get_source_code};
use crate::git_ops::{checkout_branch, checkout_commit};
use crate::types::{DetailedChanges, FileASTData};

// Compare ASTs to find differences
pub fn compare_asts(
    old_ast: &FileASTData,
    new_ast: &FileASTData,
    package_name: &str,
    file_path: &str,
    is_new_file: bool,
    is_removed_file: bool,
) -> DetailedChanges {
    let mut changes = DetailedChanges::new(file_path.to_string());

    // Handle special cases for new or removed files
    if is_new_file {
        // For new files, all elements are considered "added"
        // Extract all functions from the new AST
        for (name, func_decl) in &new_ast.functions {
            let code = format_node(func_decl);
            changes.added_functions.push(vec![name.clone(), code]);
        }

        // Extract all types from the new AST
        for (name, type_spec) in &new_ast.types {
            let code = format_node(type_spec);
            changes.added_types.push(vec![name.clone(), code]);
        }

        // Extract all interfaces from the new AST
        for (name, interface_spec) in &new_ast.interfaces {
            let code = format_node(interface_spec);
            changes.added_interfaces.push(vec![name.clone(), code]);
        }

        // Extract all methods from the new AST
        for (name, (_, method_decl)) in &new_ast.methods {
            let code = format_node(method_decl);
            changes.added_methods.push(vec![name.clone(), code]);
        }

        return changes;
    }

    if is_removed_file {
        // For removed files, all elements are considered "deleted"
        // Extract all functions from the old AST
        for (name, func_decl) in &old_ast.functions {
            let code = format_node(func_decl);
            changes.deleted_functions.push(vec![name.clone(), code]);
        }

        // Extract all types from the old AST
        for (name, type_spec) in &old_ast.types {
            let code = format_node(type_spec);
            changes.deleted_types.push(vec![name.clone(), code]);
        }

        // Extract all interfaces from the old AST
        for (name, interface_spec) in &old_ast.interfaces {
            let code = format_node(interface_spec);
            changes.deleted_interfaces.push(vec![name.clone(), code]);
        }

        // Extract all methods from the old AST
        for (name, (_, method_decl)) in &old_ast.methods {
            let code = format_node(method_decl);
            changes.deleted_methods.push(vec![name.clone(), code]);
        }

        return changes;
    }

    // Normal case - file exists in both versions
    // Compare functions
    changes.added_functions = find_added_func_elements(&old_ast.functions, &new_ast.functions);
    changes.modified_functions = find_modified_func_elements(&old_ast.functions, &new_ast.functions);
    changes.deleted_functions = find_deleted_func_elements(&old_ast.functions, &new_ast.functions);

    // Compare types
    changes.added_types = find_added_type_elements(&old_ast.types, &new_ast.types);
    changes.modified_types = find_modified_type_elements(&old_ast.types, &new_ast.types);
    changes.deleted_types = find_deleted_type_elements(&old_ast.types, &new_ast.types);

    // Compare interfaces
    changes.added_interfaces = find_added_trait_elements(&old_ast.interfaces, &new_ast.interfaces);
    changes.modified_interfaces = find_modified_trait_elements(&old_ast.interfaces, &new_ast.interfaces);
    changes.deleted_interfaces = find_deleted_trait_elements(&old_ast.interfaces, &new_ast.interfaces);

    // Compare methods
    changes.added_methods = find_added_method_elements(&old_ast.methods, &new_ast.methods);
    changes.modified_methods = find_modified_method_elements(&old_ast.methods, &new_ast.methods);
    changes.deleted_methods = find_deleted_method_elements(&old_ast.methods, &new_ast.methods);

    changes
}

// Find elements present in new but not in old (for functions)
fn find_added_func_elements(
    old_map: &HashMap<String, ItemFn>,
    new_map: &HashMap<String, ItemFn>,
) -> Vec<Vec<String>> {
    let mut added = Vec::new();

    for (name, new_node) in new_map {
        if !old_map.contains_key(name) {
            let code = format_node(new_node);
            added.push(vec![name.clone(), code]);
        }
    }

    added
}

// Find elements present in both but with different code (for functions)
fn find_modified_func_elements(
    old_map: &HashMap<String, ItemFn>,
    new_map: &HashMap<String, ItemFn>,
) -> Vec<Vec<String>> {
    let mut modified = Vec::new();

    for (name, old_node) in old_map {
        if let Some(new_node) = new_map.get(name) {
            let old_code = format_node(old_node);
            let new_code = format_node(new_node);
            
            if old_code != new_code {
                modified.push(vec![name.clone(), old_code, new_code]);
            }
        }
    }

    modified
}

// Find elements present in old but not in new (for functions)
fn find_deleted_func_elements(
    old_map: &HashMap<String, ItemFn>,
    new_map: &HashMap<String, ItemFn>,
) -> Vec<Vec<String>> {
    let mut deleted = Vec::new();

    for (name, old_node) in old_map {
        if !new_map.contains_key(name) {
            let code = format_node(old_node);
            deleted.push(vec![name.clone(), code]);
        }
    }

    deleted
}

// Find elements present in new but not in old (for types)
fn find_added_type_elements(
    old_map: &HashMap<String, Item>,
    new_map: &HashMap<String, Item>,
) -> Vec<Vec<String>> {
    let mut added = Vec::new();

    for (name, new_node) in new_map {
        if !old_map.contains_key(name) {
            let code = format_node(new_node);
            added.push(vec![name.clone(), code]);
        }
    }

    added
}

// Find elements present in both but with different code (for types)
fn find_modified_type_elements(
    old_map: &HashMap<String, Item>,
    new_map: &HashMap<String, Item>,
) -> Vec<Vec<String>> {
    let mut modified = Vec::new();

    for (name, old_node) in old_map {
        if let Some(new_node) = new_map.get(name) {
            let old_code = format_node(old_node);
            let new_code = format_node(new_node);
            
            if old_code != new_code {
                modified.push(vec![name.clone(), old_code, new_code]);
            }
        }
    }

    modified
}

// Find elements present in old but not in new (for types)
fn find_deleted_type_elements(
    old_map: &HashMap<String, Item>,
    new_map: &HashMap<String, Item>,
) -> Vec<Vec<String>> {
    let mut deleted = Vec::new();

    for (name, old_node) in old_map {
        if !new_map.contains_key(name) {
            let code = format_node(old_node);
            deleted.push(vec![name.clone(), code]);
        }
    }

    deleted
}

// Find elements present in new but not in old (for traits/interfaces)
fn find_added_trait_elements(
    old_map: &HashMap<String, ItemTrait>,
    new_map: &HashMap<String, ItemTrait>,
) -> Vec<Vec<String>> {
    let mut added = Vec::new();

    for (name, new_node) in new_map {
        if !old_map.contains_key(name) {
            let code = format_node(new_node);
            added.push(vec![name.clone(), code]);
        }
    }

    added
}

// Find elements present in both but with different code (for traits/interfaces)
fn find_modified_trait_elements(
    old_map: &HashMap<String, ItemTrait>,
    new_map: &HashMap<String, ItemTrait>,
) -> Vec<Vec<String>> {
    let mut modified = Vec::new();

    for (name, old_node) in old_map {
        if let Some(new_node) = new_map.get(name) {
            let old_code = format_node(old_node);
            let new_code = format_node(new_node);
            
            if old_code != new_code {
                modified.push(vec![name.clone(), old_code, new_code]);
            }
        }
    }

    modified
}

// Find elements present in old but not in new (for traits/interfaces)
fn find_deleted_trait_elements(
    old_map: &HashMap<String, ItemTrait>,
    new_map: &HashMap<String, ItemTrait>,
) -> Vec<Vec<String>> {
    let mut deleted = Vec::new();

    for (name, old_node) in old_map {
        if !new_map.contains_key(name) {
            let code = format_node(old_node);
            deleted.push(vec![name.clone(), code]);
        }
    }

    deleted
}

// Find elements present in new but not in old (for methods)
fn find_added_method_elements(
    old_map: &HashMap<String, (ItemImpl, ItemFn)>,
    new_map: &HashMap<String, (ItemImpl, ItemFn)>,
) -> Vec<Vec<String>> {
    let mut added = Vec::new();

    for (name, (_, new_node)) in new_map {
        if !old_map.contains_key(name) {
            let code = format_node(new_node);
            added.push(vec![name.clone(), code]);
        }
    }

    added
}

// Find elements present in both but with different code (for methods)
fn find_modified_method_elements(
    old_map: &HashMap<String, (ItemImpl, ItemFn)>,
    new_map: &HashMap<String, (ItemImpl, ItemFn)>,
) -> Vec<Vec<String>> {
    let mut modified = Vec::new();

    for (name, (_, old_node)) in old_map {
        if let Some((_, new_node)) = new_map.get(name) {
            let old_code = format_node(old_node);
            let new_code = format_node(new_node);
            
            if old_code != new_code {
                modified.push(vec![name.clone(), old_code, new_code]);
            }
        }
    }

    modified
}

// Find elements present in old but not in new (for methods)
fn find_deleted_method_elements(
    old_map: &HashMap<String, (ItemImpl, ItemFn)>,
    new_map: &HashMap<String, (ItemImpl, ItemFn)>,
) -> Vec<Vec<String>> {
    let mut deleted = Vec::new();

    for (name, (_, old_node)) in old_map {
        if !new_map.contains_key(name) {
            let code = format_node(old_node);
            deleted.push(vec![name.clone(), code]);
        }
    }

    deleted
}

// Process all Rust files with minimized Git checkouts
pub fn process_rust_files(
    rust_files: &[String],
    local_repo_path: &str,
    branch_name: &str,
    current_commit: &str,
    new_file_map: &HashMap<String, bool>,
    deleted_file_map: &HashMap<String, bool>,
) -> Vec<DetailedChanges> {
    let mut all_changes = Vec::new();

    // Maps to store ASTs from both commits
    let mut branch_asts = HashMap::new();
    let mut current_asts = HashMap::new();

    // Step 1: Checkout branch commit and extract ASTs for all files
    if let Err(e) = checkout_branch(branch_name, local_repo_path) {
        println!("Error checking out branch {}: {}", branch_name, e);
        return all_changes;
    }
    println!("Successfully checked out branch {}", branch_name);

    // Process all files in the branch commit (except new files)
    for go_file in rust_files {
        if !new_file_map.contains_key(go_file) {
            let full_path = Path::new(local_repo_path).join(go_file);
            match extract_file_ast(full_path.to_str().unwrap_or("")) {
                Ok(ast) => {
                    branch_asts.insert(go_file.clone(), ast);
                },
                Err(e) => {
                    println!("Error parsing AST for {} in branch: {}", go_file, e);
                    // Create an empty AST if we couldn't parse the file
                    branch_asts.insert(go_file.clone(), FileASTData::empty(go_file.clone()));
                }
            }
        }
    }

    // Step 2: Checkout current commit and extract ASTs for all files
    if let Err(e) = checkout_commit(current_commit, local_repo_path) {
        println!("Error checking out commit {}: {}", current_commit, e);
        
        // Try alternative checkout method
        if let Err(e) = checkout_commit(&format!("{}^{{commit}}", current_commit), local_repo_path) {
            println!("Error checking out commit using alternative method: {}", e);
            return all_changes;
        }
        
        println!("Successfully checked out commit using alternative method.");
    } else {
        println!("Successfully checked out commit {}", current_commit);
    }

    // Process all files in the current commit (except deleted files)
    for go_file in rust_files {
        if !deleted_file_map.contains_key(go_file) {
            let full_path = Path::new(local_repo_path).join(go_file);
            match extract_file_ast(full_path.to_str().unwrap_or("")) {
                Ok(ast) => {
                    current_asts.insert(go_file.clone(), ast);
                },
                Err(e) => {
                    println!("Error parsing AST for {} in current commit: {}", go_file, e);
                    // Create an empty AST if we couldn't parse the file
                    current_asts.insert(go_file.clone(), FileASTData::empty(go_file.clone()));
                }
            }
        }
    }

    // Step 3: Compare all ASTs and collect changes
    for go_file in rust_files {
        // Extract package name for the module name
        let package_name;
        
        if new_file_map.contains_key(go_file) {
            // For new files, extract package from current commit's AST
            let full_path = Path::new(local_repo_path).join(go_file);
            package_name = extract_module_name(full_path.to_str().unwrap_or(""));
        } else if deleted_file_map.contains_key(go_file) {
            // For deleted files, we can't reliably get package name from the file
            // Use directory name as fallback
            package_name = Path::new(go_file)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();
        } else {
            // For modified files, use current commit's package name
            let full_path = Path::new(local_repo_path).join(go_file);
            package_name = extract_module_name(full_path.to_str().unwrap_or(""));
        }

        // Initialize old and new ASTs
        let old_ast;
        let new_ast;

        if new_file_map.contains_key(go_file) {
            // For new files: empty old AST, new AST from current commit
            old_ast = FileASTData::empty(go_file.clone());
            new_ast = current_asts.get(go_file).cloned().unwrap_or_else(|| FileASTData::empty(go_file.clone()));
            println!("File {} is new", go_file);
        } else if deleted_file_map.contains_key(go_file) {
            // For deleted files: old AST from branch, empty new AST
            old_ast = branch_asts.get(go_file).cloned().unwrap_or_else(|| FileASTData::empty(go_file.clone()));
            new_ast = FileASTData::empty(go_file.clone());
            println!("File {} has been deleted", go_file);
        } else {
            // For modified files: both ASTs
            old_ast = branch_asts.get(go_file).cloned().unwrap_or_else(|| FileASTData::empty(go_file.clone()));
            new_ast = current_asts.get(go_file).cloned().unwrap_or_else(|| FileASTData::empty(go_file.clone()));
        }

        // Compare ASTs and collect changes
        let changes = compare_asts(
            &old_ast,
            &new_ast,
            &package_name,
            go_file,
            new_file_map.contains_key(go_file),
            deleted_file_map.contains_key(go_file),
        );
        
        if changes.has_changes() {
            all_changes.push(changes);
        }
    }

    all_changes
}