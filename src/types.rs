// src/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn::{Item, ItemFn, ItemImpl, ItemTrait};

// SourceLocation captures position information of a declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub file_name: String,
}

// TypedLiteral represents a literal value with its type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedLiteral {
    pub type_name: String,
    pub value: String,
}

// CalledFunctionChanges captures the granular changes in function calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalledFunctionChanges {
    pub added_functions: Vec<String>,
    pub removed_functions: Vec<String>,
    pub added_literals: Vec<TypedLiteral>,
    pub removed_literals: Vec<TypedLiteral>,
    pub old_function_src_loc: SourceLocation,
    pub new_function_src_loc: SourceLocation,
}

// NamedCode represents a named code entity with its source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedCode {
    pub name: String,
    pub code: String,
}

// ModifiedCode represents a code entity that has been modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedCode {
    pub name: String,
    pub old_code: String,
    pub new_code: String,
}

// DetailedChanges captures all types of declarations that can change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedChanges {
    pub module_name: String,
    pub added_functions: Vec<Vec<String>>,
    pub modified_functions: Vec<Vec<String>>,
    pub deleted_functions: Vec<Vec<String>>,
    pub added_types: Vec<Vec<String>>,
    pub modified_types: Vec<Vec<String>>,
    pub deleted_types: Vec<Vec<String>>,
    pub added_interfaces: Vec<Vec<String>>,
    pub modified_interfaces: Vec<Vec<String>>,
    pub deleted_interfaces: Vec<Vec<String>>,
    pub added_methods: Vec<Vec<String>>,
    pub modified_methods: Vec<Vec<String>>,
    pub deleted_methods: Vec<Vec<String>>,
}

impl DetailedChanges {
    pub fn new(module_name: String) -> Self {
        DetailedChanges {
            module_name,
            added_functions: Vec::new(),
            modified_functions: Vec::new(),
            deleted_functions: Vec::new(),
            added_types: Vec::new(),
            modified_types: Vec::new(),
            deleted_types: Vec::new(),
            added_interfaces: Vec::new(),
            modified_interfaces: Vec::new(),
            deleted_interfaces: Vec::new(),
            added_methods: Vec::new(),
            modified_methods: Vec::new(),
            deleted_methods: Vec::new(),
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.added_functions.is_empty() ||
        !self.modified_functions.is_empty() ||
        !self.deleted_functions.is_empty() ||
        !self.added_types.is_empty() ||
        !self.modified_types.is_empty() ||
        !self.deleted_types.is_empty() ||
        !self.added_interfaces.is_empty() ||
        !self.modified_interfaces.is_empty() ||
        !self.deleted_interfaces.is_empty() ||
        !self.added_methods.is_empty() ||
        !self.modified_methods.is_empty() ||
        !self.deleted_methods.is_empty()
    }
}

// FileASTData stores AST information for a Rust file
#[derive(Debug,Clone)]
pub struct FileASTData {
    pub functions: HashMap<String, ItemFn>,
    pub types: HashMap<String, Item>,         // Struct, Enum, Type Alias
    pub interfaces: HashMap<String, ItemTrait>, // Traits in Rust
    pub methods: HashMap<String, (ItemImpl, ItemFn)>, // impl methods
    pub file_content: String,
    pub file_path: String,
}

impl FileASTData {
    pub fn new(file_path: String, file_content: String) -> Self {
        FileASTData {
            functions: HashMap::new(),
            types: HashMap::new(),
            interfaces: HashMap::new(),
            methods: HashMap::new(),
            file_content,
            file_path,
        }
    }
    
    pub fn empty(file_path: String) -> Self {
        FileASTData {
            functions: HashMap::new(),
            types: HashMap::new(),
            interfaces: HashMap::new(),
            methods: HashMap::new(),
            file_content: String::new(),
            file_path,
        }
    }
}

// Structure for holding function call visitor data
pub struct FunctionCallVisitor {
    pub calls: Vec<String>,
}

// Structure for holding literal visitor data
pub struct LiteralVisitor {
    pub literals: Vec<TypedLiteral>,
}