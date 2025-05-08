// src/git_ops.rs
use std::path::Path;
use std::process::Command;

// Clone a Git repository if it doesn't exist locally
pub fn clone_repo(repo_url: &str, branch_name: &str, local_path: &str) {
    let path = Path::new(local_path);
    
    if !path.exists() {
        println!("Cloning repository {} to {}", repo_url, local_path);
        
        let output = Command::new("git")
            .args(&["clone", repo_url, local_path])
            .output()
            .expect("Failed to execute git clone command");
            
        if !output.status.success() {
            eprintln!("Error cloning repository: {}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }
    } else {
        println!("Repository already cloned.");
        
        // Set the remote URL
        let output_remote = Command::new("git")
            .args(&["remote", "set-url", "origin", repo_url])
            .current_dir(local_path)
            .output()
            .expect("Failed to set remote URL");
            
        if output_remote.status.success() {
            println!("Successfully set origin remote url");
        } else {
            println!("Warning: Failed to set remote URL: {}", String::from_utf8_lossy(&output_remote.stderr));
        }
        
        // List all branches for debugging
        let output_branches = Command::new("git")
            .args(&["branch", "--all"])
            .current_dir(local_path)
            .output()
            .expect("Failed to list branches");
            
        if output_branches.status.success() {
            println!("Successfully fetched all branches \n{}", String::from_utf8_lossy(&output_branches.stdout));
        } else {
            println!("Warning: Failed to list branches: {}", String::from_utf8_lossy(&output_branches.stderr));
        }
        
        // Fetch the latest changes
        let output_fetch = Command::new("git")
            .args(&["fetch"])
            .current_dir(local_path)
            .output()
            .expect("Failed to fetch latest changes");
            
        if output_fetch.status.success() {
            println!("Successfully fetched latest changes. {}", String::from_utf8_lossy(&output_fetch.stdout));
        } else {
            println!("Warning: Failed to fetch latest changes: {}", String::from_utf8_lossy(&output_fetch.stderr));
        }
    }
}

// Get a list of files that are new in the current commit compared to the branch
pub fn get_new_files(branch_name: &str, new_commit: &str, local_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(&["diff", "--name-only", "--diff-filter=A", branch_name, new_commit])
        .current_dir(local_path)
        .output()
        .map_err(|e| format!("Failed to execute git diff command: {}", e))?;
        
    if !output.status.success() {
        return Err(format!(
            "Error getting new files between {} and {}: {}",
            branch_name,
            new_commit,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let files_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    let files = if files_str.is_empty() {
        Vec::new()
    } else {
        files_str.split('\n').map(|s| s.to_string()).collect()
    };
    
    println!(
        "Detected {} new files added between {} and {}",
        files.len(),
        branch_name,
        new_commit
    );
    
    Ok(files)
}

// Get a list of files that were deleted in the current commit compared to the branch
pub fn get_deleted_files(branch_name: &str, new_commit: &str, local_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(&["diff", "--name-only", "--diff-filter=D", branch_name, new_commit])
        .current_dir(local_path)
        .output()
        .map_err(|e| format!("Failed to execute git diff command: {}", e))?;
        
    if !output.status.success() {
        return Err(format!(
            "Error getting deleted files between {} and {}: {}",
            branch_name,
            new_commit,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let files_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    let files = if files_str.is_empty() {
        Vec::new()
    } else {
        files_str.split('\n').map(|s| s.to_string()).collect()
    };
    
    println!(
        "Detected {} files deleted between {} and {}",
        files.len(),
        branch_name,
        new_commit
    );
    
    Ok(files)
}

// Get a list of files that have changed between the current state and the branch
pub fn get_changed_files(branch_name: &str, local_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(&["diff", "--name-only", branch_name])
        .current_dir(local_path)
        .output()
        .map_err(|e| format!("Failed to execute git diff command: {}", e))?;
        
    if !output.status.success() {
        return Err(format!(
            "Error getting direct diff for {}: {}",
            branch_name,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let files_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    let files = if files_str.is_empty() {
        Vec::new()
    } else {
        files_str.split('\n').map(|s| s.to_string()).collect()
    };
    
    Ok(files)
}

// Checkout a specific branch
pub fn checkout_branch(branch_name: &str, local_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(&["checkout", branch_name])
        .current_dir(local_path)
        .output()
        .map_err(|e| format!("Failed to execute git checkout command: {}", e))?;
        
    if !output.status.success() {
        return Err(format!(
            "Failed to checkout branch {}: {}",
            branch_name,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

// Checkout a specific commit
pub fn checkout_commit(commit: &str, local_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(&["checkout", commit])
        .current_dir(local_path)
        .output()
        .map_err(|e| format!("Failed to execute git checkout command: {}", e))?;
        
    if !output.status.success() {
        return Err(format!(
            "Failed to checkout commit {}: {}",
            commit,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}