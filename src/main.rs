mod lexer;
mod parser;
mod ast;
mod form_parser;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let raw: Vec<String> = env::args().collect();

    // Parse --target <value> flag; everything else is positional.
    let mut target = "cursive".to_string();
    let mut positional: Vec<String> = Vec::new();
    let mut i = 1;
    while i < raw.len() {
        if raw[i] == "--target" && i + 1 < raw.len() {
            target = raw[i + 1].clone();
            i += 2;
        } else {
            positional.push(raw[i].clone());
            i += 1;
        }
    }

    if positional.len() != 1 {
        eprintln!("Usage: {} [--target cursive|egui|web] <input.vb>", raw[0]);
        std::process::exit(1);
    }
    let input_path = &positional[0];

    let input = fs::read_to_string(input_path)
        .unwrap_or_else(|_| panic!("Cannot read file: {}", input_path));

    if target == "web" {
        // Web target: emit two files next to the source.
        if !contains_form_block(&input) {
            eprintln!("✘ --target web requires a Form block");
            std::process::exit(1);
        }
        match form_parser::transpile_web_form_file(&input) {
            Ok((rust_code, json_layout)) => {
                let stem = Path::new(input_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("form");
                let dir = Path::new(input_path).parent().unwrap_or(Path::new("."));
                let rs_path  = dir.join(format!("{}.wasm.rs", stem));
                let json_path = dir.join(format!("{}.layout.json", stem));
                fs::write(&rs_path, &rust_code).expect("write .wasm.rs");
                fs::write(&json_path, &json_layout).expect("write .layout.json");
                eprintln!("Generated {} and {}", rs_path.display(), json_path.display());
            }
            Err(e) => {
                eprintln!("✘ {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // TUI / egui targets — print to stdout.
    let result = if contains_form_block(&input) {
        form_parser::transpile_form_file(&input, &target)
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
