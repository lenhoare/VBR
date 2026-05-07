mod lexer;
mod parser;
mod ast;
mod form_parser;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.vb>", args[0]);
        std::process::exit(1);
    }

    let input = fs::read_to_string(&args[1])
        .unwrap_or_else(|_| panic!("Cannot read file: {}", args[1]));

    // If the file contains a Form block, use the form transpiler.
    // Otherwise fall back to the simple line-by-line transpiler.
    let result = if contains_form_block(&input) {
        form_parser::transpile_form_file(&input)
    } else {
        transpile_simple(&input)
    };

    match result {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("✘ {}", e);
            std::process::exit(1);
        }
    }
}

fn contains_form_block(input: &str) -> bool {
    input.lines().any(|l| {
        let t = l.trim();
        t.starts_with("Form ") && t[5..].trim_start().starts_with('"')
    })
}

// ---------------------------------------------------------------------------
// Simple line-by-line transpiler (non-form VBR files)
// ---------------------------------------------------------------------------

fn transpile_simple(input: &str) -> Result<String, String> {
    let mut output = String::from("// Transpiled VBR code\n");
    output.push_str("use std::collections::HashMap;\n\n");
    output.push_str("fn main() {\n");

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        if trimmed.starts_with("Dim ") {
            output.push_str(&format!("    {}\n", convert_dim(trimmed)));
        } else if trimmed.starts_with("Function ") {
            output.push_str(&format!("    {}\n", convert_function(trimmed)));
        } else if trimmed.starts_with("If ") {
            output.push_str(&format!("    {}\n", convert_if(trimmed)));
        } else if trimmed.starts_with("For ") {
            output.push_str(&format!("    {}\n", convert_for(trimmed)));
        } else if trimmed.contains('=') && !trimmed.contains("==") {
            output.push_str(&format!("    {};\n", trimmed));
        } else {
            output.push_str(&format!("    // {} (needs conversion)\n", trimmed));
        }
    }

    output.push_str("}\n");
    Ok(output)
}

fn convert_dim(line: &str) -> String {
    let parts: Vec<&str> = line[4..].split_whitespace().collect();
    if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("as") {
        let var_name = parts[0];
        let var_type = parts[2..].join(" ");
        let rust_type = vbr_type_to_rust(&var_type);
        format!("let {}: {};", var_name, rust_type)
    } else {
        line.to_string()
    }
}

fn convert_function(line: &str) -> String {
    let name_start = line.find(' ').unwrap_or(line.len());
    let name_end   = line.find('(').unwrap_or(line.len());
    if name_end > name_start {
        let name = &line[name_start + 1..name_end];
        format!("fn {}() {{\n        // TODO: implement\n    }}", name)
    } else {
        line.to_string()
    }
}

fn convert_if(line: &str) -> String {
    let cond_start = line.find(' ').unwrap_or(line.len());
    let cond_end   = line.find(" Then").unwrap_or(line.len());
    let condition  = &line[cond_start + 1..cond_end];
    format!("if {} {{\n        // Then branch\n    }}", condition)
}

fn convert_for(line: &str) -> String {
    let parts: Vec<&str> = line[4..].split_whitespace().collect();
    if parts.len() >= 5 && parts[1] == "=" && parts[3].eq_ignore_ascii_case("to") {
        let var   = parts[0];
        let start = parts[2];
        let end   = parts[4];
        format!("for {} in {}..={} {{\n        // loop body\n    }}", var, start, end)
    } else {
        line.to_string()
    }
}

fn vbr_type_to_rust(t: &str) -> &str {
    match t.trim() {
        "Integer"  => "i32",
        "Long"     => "i32",
        "LongLong" => "i64",
        "Single"   => "f32",
        "Double"   => "f64",
        "Boolean"  => "bool",
        "Bool"     => "bool",
        "Byte"     => "u8",
        "String"   => "String",
        other      => other,
    }
}
