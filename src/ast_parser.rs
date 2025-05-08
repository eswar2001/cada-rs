// src/ast_parser.rs
use std::fs;
use std::path::Path;
use proc_macro2::Span;
use syn::{parse_file, visit::{self, Visit}, File, Item, ItemFn, ItemImpl};
use crate::types::{FileASTData, SourceLocation, TypedLiteral, FunctionCallVisitor, LiteralVisitor};

// Extract the module name from a Rust file
pub fn extract_module_name(file_path: &str) -> String {
    // Try to parse the file to extract the module name
    if let Ok(content) = fs::read_to_string(file_path) {
        if let Ok(file) = parse_file(&content) {
            for item in file.items {
                if let Item::Mod(module) = item {
                    return module.ident.to_string();
                }
            }
        }
    }
    
    // If we can't find a module declaration, use the directory name
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if let Some(dir_name) = parent.file_name() {
            if let Some(name) = dir_name.to_str() {
                return name.to_string();
            }
        }
    }
    
    // Last resort: use "unknown"
    "unknown".to_string()
}

// Extract AST data from a Rust file
pub fn extract_file_ast(file_path: &str) -> Result<FileASTData, String> {
    println!("Reading file: {}", file_path);
    
    // Read file content
    let file_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                println!("File does not exist at this commit: {}", file_path);
                return Err(format!("File not found: {}", e));
            }
            return Err(format!("File couldn't be read: {}", e));
        }
    };
    
    println!("File size: {} bytes", file_content.len());
    
    // Parse file to AST
    let file = match parse_file(&file_content) {
        Ok(ast) => ast,
        Err(e) => return Err(format!("Parsing error: {}", e)),
    };
    
    // Initialize AST data
    let mut ast_data = FileASTData::new(file_path.to_string(), file_content);
    
    // Process all items in the file
    process_file_items(&file, &mut ast_data);
    
    Ok(ast_data)
}

// Process all items in a Rust file
fn process_file_items(file: &File, ast_data: &mut FileASTData) {
    for item in &file.items {
        match item {
            Item::Fn(func) => {
                // Regular function
                let func_name = func.sig.ident.to_string();
                ast_data.functions.insert(func_name.clone(), func.clone());
                println!("Extracted function {} from {}", func_name, ast_data.file_path);
            },
            Item::Impl(impl_block) => {
                // Methods inside impl blocks
                process_impl_block(impl_block, ast_data);
            },
            Item::Trait(trait_def) => {
                // Trait definition (interface in Rust)
                let trait_name = trait_def.ident.to_string();
                ast_data.interfaces.insert(trait_name.clone(), trait_def.clone());
                println!("Extracted trait {} from {}", trait_name, ast_data.file_path);
            },
            Item::Struct(struct_def) => {
                // Struct definition
                let struct_name = struct_def.ident.to_string();
                ast_data.types.insert(struct_name.clone(), Item::Struct(struct_def.clone()));
                println!("Extracted struct {} from {}", struct_name, ast_data.file_path);
            },
            Item::Enum(enum_def) => {
                // Enum definition
                let enum_name = enum_def.ident.to_string();
                ast_data.types.insert(enum_name.clone(), Item::Enum(enum_def.clone()));
                println!("Extracted enum {} from {}", enum_name, ast_data.file_path);
            },
            Item::Type(type_alias) => {
                // Type alias
                let type_name = type_alias.ident.to_string();
                ast_data.types.insert(type_name.clone(), Item::Type(type_alias.clone()));
                println!("Extracted type alias {} from {}", type_name, ast_data.file_path);
            },
            _ => {} // Ignore other items
        }
    }
}

// Process methods inside impl blocks
fn process_impl_block(impl_block: &ItemImpl, ast_data: &mut FileASTData) {
    // Get the type name for this impl block
    let type_name = match &*impl_block.self_ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                return; // Can't determine type name
            }
        },
        _ => return, // Can't determine type name
    };
    
    // Process all items inside the impl block
    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            let full_name = format!("{}.{}", type_name, method_name);
            
            // Convert impl method to a standalone function
            let fn_item = ItemFn {
                attrs: method.attrs.clone(),
                vis: method.vis.clone(),
                sig: method.sig.clone(),
                block: Box::new(method.block.clone()),
            };
            
            ast_data.methods.insert(full_name.clone(), (impl_block.clone(), fn_item));
            println!("Extracted method {} from {}", full_name, ast_data.file_path);
        }
    }
}

// Extract source code from the original content
pub fn get_source_code(span: Span, file_content: &str) -> String {
    let start: proc_macro2::LineColumn = span.start();
    let end = span.end();
    
    // Safety check to avoid out-of-bounds access
    let content_bytes = file_content.as_bytes();
    if start.line == 0 || end.line == 0 || 
       start.line > file_content.lines().count() || 
       end.line > file_content.lines().count() {
        return String::new();
    }
    
    // Find the byte offsets
    let mut start_offset = 0;
    let mut end_offset = 0;
    let mut current_line = 1;
    let mut current_column = 1;
    
    for (i, byte) in content_bytes.iter().enumerate() {
        if current_line == start.line && current_column == start.column {
            start_offset = i;
        }
        
        if current_line == end.line && current_column == end.column {
            end_offset = i;
            break;
        }
        
        // Update line and column counters
        if *byte == b'\n' {
            current_line += 1;
            current_column = 1;
        } else {
            current_column += 1;
        }
    }
    
    if start_offset < end_offset && end_offset <= content_bytes.len() {
        String::from_utf8_lossy(&content_bytes[start_offset..end_offset]).to_string()
    } else {
        String::new()
    }
}

// Get source location from a span
pub fn get_source_location(span: Span, filename: &str) -> SourceLocation {
    let start = span.start();
    let end = span.end();
    
    SourceLocation {
        start_line: start.line,
        start_col: start.column,
        end_line: end.line,
        end_col: end.column,
        file_name: filename.to_string(),
    }
}

// Extract function calls from a function
pub fn extract_function_calls(func: &ItemFn) -> Vec<String> {
    let mut visitor = FunctionCallVisitor {
        calls: Vec::new(),
    };
    
    visitor.visit_item_fn(func);
    
    visitor.calls
}

// Extract literals from a function
pub fn extract_literals(func: &ItemFn) -> Vec<TypedLiteral> {
    let mut visitor = LiteralVisitor {
        literals: Vec::new(),
    };
    
    visitor.visit_item_fn(func);
    
    visitor.literals
}

// Remove duplicates from a vector of strings
pub fn remove_duplicates(strings: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    
    for s in strings {
        if !seen.contains(&s) {
            seen.insert(s.clone());
            result.push(s);
        }
    }
    
    result
}

// Format an AST node as a string
pub fn format_node<T: syn::parse::Parse + quote::ToTokens>(node: &T) -> String {
    // Since node already implements ToTokens, we can directly use quote! on it
    quote::quote!(#node).to_string()
}

// Implementation for the function call visitor
impl<'ast> Visit<'ast> for FunctionCallVisitor {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        // Extract function name if possible
        if let syn::Expr::Path(expr_path) = &*call.func {
            let path = &expr_path.path;
            if let Some(last_segment) = path.segments.last() {
                self.calls.push(last_segment.ident.to_string());
            }
        } else if let syn::Expr::Field(expr_field) = &*call.func {
            // Handle method calls (struct.method())
            if let syn::Expr::Path(base_path) = &*expr_field.base {
                if let Some(base_segment) = base_path.path.segments.last() {
                    let base_name = base_segment.ident.to_string();
                    let method_name = &expr_field.member;
                    self.calls.push(format!("{}.{:?}", base_name, method_name));
                }
            }
        }
        
        // Continue visiting children
        visit::visit_expr_call(self, call);
    }
    
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        // Handle method calls (obj.method())
        let method_name = call.method.to_string();
        
        // Try to get the type/variable name
        let mut base_name = String::new();
        if let syn::Expr::Path(base_path) = &*call.receiver {
            if let Some(base_segment) = base_path.path.segments.last() {
                base_name = base_segment.ident.to_string();
            }
        }
        
        if !base_name.is_empty() {
            self.calls.push(format!("{}.{}", base_name, method_name));
        } else {
            self.calls.push(method_name);
        }
        
        // Continue visiting children
        visit::visit_expr_method_call(self, call);
    }
}

// Implementation for the literal visitor
impl<'ast> Visit<'ast> for LiteralVisitor {
    fn visit_lit(&mut self, lit: &'ast syn::Lit) {
        match lit {
            syn::Lit::Str(lit_str) => {
                self.literals.push(TypedLiteral {
                    type_name: "STRING".to_string(),
                    value: lit_str.value(),
                });
            },
            syn::Lit::Int(lit_int) => {
                self.literals.push(TypedLiteral {
                    type_name: "INT".to_string(),
                    value: lit_int.base10_digits().to_string(),
                });
            },
            syn::Lit::Float(lit_float) => {
                self.literals.push(TypedLiteral {
                    type_name: "FLOAT".to_string(),
                    value: lit_float.base10_digits().to_string(),
                });
            },
            syn::Lit::Bool(lit_bool) => {
                self.literals.push(TypedLiteral {
                    type_name: "BOOL".to_string(),
                    value: lit_bool.value.to_string(),
                });
            },
            syn::Lit::Char(lit_char) => {
                self.literals.push(TypedLiteral {
                    type_name: "CHAR".to_string(),
                    value: lit_char.value().to_string(),
                });
            },
            syn::Lit::Byte(lit_byte) => {
                self.literals.push(TypedLiteral {
                    type_name: "BYTE".to_string(),
                    value: lit_byte.value().to_string(),
                });
            },
            syn::Lit::ByteStr(_) => {
                self.literals.push(TypedLiteral {
                    type_name: "BYTE_STR".to_string(),
                    value: "byte-string-literal".to_string(),
                });
            },
            syn::Lit::Verbatim(_) => {
                self.literals.push(TypedLiteral {
                    type_name: "VERBATIM".to_string(),
                    value: "verbatim-literal".to_string(),
                });
            },
            &_ => {

            }
        }
        
        // Continue visiting children
        visit::visit_lit(self, lit);
    }
    
    fn visit_expr_struct(&mut self, expr: &'ast syn::ExprStruct) {
        // Handle struct literals
        if let Some(path_segment) = expr.path.segments.last() {
            let struct_name = path_segment.ident.to_string();
            self.literals.push(TypedLiteral {
                type_name: format!("CompositeLit:{}", struct_name),
                value: "composite-literal".to_string(),
            });
        }
        
        // Continue visiting children
        visit::visit_expr_struct(self, expr);
    }
    
    fn visit_expr_array(&mut self, expr: &'ast syn::ExprArray) {
        // Handle array literals
        self.literals.push(TypedLiteral {
            type_name: "CompositeLit:Array".to_string(),
            value: "composite-literal".to_string(),
        });
        
        // Continue visiting children
        visit::visit_expr_array(self, expr);
    }
}