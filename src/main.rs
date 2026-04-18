mod lexer;
mod parser;
mod ast;

use std::env;
use std::fs;

fn transpile_simple(input: &str) -> Result<String, String> {
    // Simple regex-based transpiler for demonstration
    let mut output = String::from("// Transpiled VBR code\n");
    output.push_str("use std::collections::HashMap;\n\n");
    output.push_str("fn main() {\n");
    
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        // Handle Dim statements
        if trimmed.starts_with("Dim ") {
            let converted = convert_dim(trimmed);
            output.push_str(&format!("    {}\n", converted));
        }
        // Handle Function
        else if trimmed.starts_with("Function ") {
            let converted = convert_function(trimmed);
            output.push_str(&format!("    {}\n", converted));
        }
        // Handle If
        else if trimmed.starts_with("If ") {
            let converted = convert_if(trimmed);
            output.push_str(&format!("    {}\n", converted));
        }
        // Handle For
        else if trimmed.starts_with("For ") {
            let converted = convert_for(trimmed);
            output.push_str(&format!("    {}\n", converted));
        }
        // Handle assignment
        else if trimmed.contains("=") && !trimmed.contains("==") {
            output.push_str(&format!("    {};\n", trimmed));
        }
        else {
            output.push_str(&format!("    // {} (needs conversion)\n", trimmed));
        }
    }
    
    output.push_str("}\n");
    Ok(output)
}

fn convert_dim(line: &str) -> String {
    let parts: Vec<&str> = line[4..].split_whitespace().collect();
    if parts.len() >= 3 && parts[1] == "As" {
        let var_name = parts[0];
        let var_type = parts[2];
        let rust_type = match var_type {
            "Integer" => "i32",
            "Long" => "i32",
            "Double" => "f64",
            "Boolean" => "bool",
            "String" => "String",
            "Byte" => "u8",
            _ => var_type,
        };
        format!("let {}: {};", var_name, rust_type)
    } else {
        line.to_string()
    }
}

fn convert_function(line: &str) -> String {
    let name_start = line.find(' ').unwrap_or(line.len());
    let name_end = line.find('(').unwrap_or(line.len());
    let name = &line[name_start+1..name_end];
    format!("fn {}() {{\n        // Function body\n    }}", name)
}

fn convert_if(line: &str) -> String {
    let cond_start = line.find(' ').unwrap_or(line.len());
    let cond_end = line.find(" Then").unwrap_or(line.len());
    let condition = &line[cond_start+1..cond_end];
    format!("if {} {{\n        // Then branch\n    }}", condition)
}

fn convert_for(line: &str) -> String {
    let parts: Vec<&str> = line[4..].split_whitespace().collect();
    if parts.len() >= 5 && parts[3] == "To" {
        let var = parts[0];
        let start = parts[2];
        let end = parts[4];
        format!("for {} in {}..={} {{\n        // Loop body\n    }}", var, start, end)
    } else {
        line.to_string()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.vb>", args[0]);
        std::process::exit(1);
    }

    let input = fs::read_to_string(&args[1]).unwrap_or_else(|_| panic!("Cannot read file"));
    match transpile_simple(&input) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("✘ Error: {}", e);
            std::process::exit(1);
        }
    }
}
