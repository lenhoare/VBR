fn transpile_statement(stmt: &Statement, indent: usize) -> String {
    let indent_str = "    ".repeat(indent);
    match stmt {
        Statement::Dim { mutable, typ, name, init } => {
            let mut_type = if *mutable { "mut " } else { "" };
            let type_str = transpile_type(typ);
            match init {
                Some(init_expr) => format!("{}let {} {}: {} = {};", indent_str, mut_type, name, type_str, transpile_expr(init_expr, indent)),
                None => format!("{}let {} {}: {};", indent_str, mut_type, name, type_str),
            }
        }
        Statement::Const { pub_, name, value } => {
            let pub_str = if *pub_ { "pub " } else { "" };
            format!("{}const {}: {} = {};", indent_str, pub_str, name, transpile_expr(value, indent))
        }
        Statement::Set { mutable, target, value } => {
            let borrow_str = if *mutable { "&mut " } else { "&" };
            format!("{}let {} = {} {};", indent_str, target, borrow_str, transpile_expr(value, indent))
        }
        Statement::If { condition, then_block, else_block } => {
            let mut output = format!("{}if {} {{\n", indent_str, transpile_expr(condition, indent + 1));
            for stmt in then_block {
                output.push_str(&transpile_statement(stmt, indent + 1));
                output.push('\n');
            }
            output.push_str(&format!("{} }} ", indent_str));
            if !else_block.is_empty() {
                output.push_str("else {");
                for stmt in else_block {
                    output.push_str(&transpile_statement(stmt, indent + 1));
                }
                output.push_str(" }");
            }
            output
        }
        Statement::Select { value, arms, else_arm } => {
            let value_str = transpile_expr(value, indent);
            let mut output = format!("{}match {} {{\n", indent_str, value_str);
            for arm in arms {
                match arm {
                    SelectArm::Value { value: ref v, ref body } => {
                        let pattern = transpile_expr(v, indent);
                        output.push_str(&format!("{}    {} => {{\n", indent_str, pattern));
                        for stmt in body {
                            output.push_str(&transpile_statement(stmt, indent + 1));
                        }
                        output.push_str(&format!("{}    }}\n", indent_str));
                    }
                    SelectArm::Range { start: ref s, end: ref e, ref body } => {
                        let start_str = transpile_expr(s, indent);
                        let end_str = transpile_expr(e, indent);
                        output.push_str(&format!("{}    {}..={} => {{\n", indent_str, start_str, end_str));
                        for stmt in body {
                            output.push_str(&transpile_statement(stmt, indent + 1));
                        }
                        output.push_str(&format!("{}    }}\n", indent_str));
                    }
                    SelectArm::Else(body) => {
                        output.push_str(&format!("{}    _ => {{\n", indent_str));
                        for stmt in body {
                            output.push_str(&transpile_statement(stmt, indent + 1));
                        }
                        output.push_str(&format!("{}    }}\n", indent_str));
                    }
                }
            }
            if let Some(ref else_arm) = else_arm {
                output.push_str(&format!("{}    _ => {{\n", indent_str));
                for stmt in else_arm {
                    output.push_str(&transpile_statement(stmt, indent + 1));
                }
                output.push_str(&format!("{}    }}\n", indent_str));
            }
            output.push_str(&format!("{} }}", indent_str));
            output
        }
        Statement::For { variable, start, end, step, body } => {
            let step_str = match step {
                Some(s) => format!(", step = {}", transpile_expr(s, indent)),
                None => String::new(),
            };
            format!("{}for {} in {}..={}{} {{\n{}\n{}}}", 
                indent_str, variable, transpile_expr(start, indent), 
                transpile_expr(end, indent), step_str,
                indent_str.repeat(indent + 1).as_str(),
                indent_str)
        }
        Statement::ForEach { variable, collection, body } => {
            format!("{}for {} in &{} {{\n{}\n{}}}",
                indent_str, variable, transpile_expr(collection, indent),
                indent_str.repeat(indent + 1).as_str(),
                indent_str)
        }
        Statement::While { condition, body } => {
            format!("{}while {} {{\n{}\n{}}}",
                indent_str, transpile_expr(condition, indent),
                indent_str.repeat(indent + 1).as_str(),
                indent_str)
        }
        Statement::DoWhile { condition, body, until } => {
            let loop_str = if *until { "until" } else { "while" };
            format!("{}loop {} {} {{\n{}\n{}}}",
                indent_str, loop_str, transpile_expr(condition, indent),
                indent_str.repeat(indent + 1).as_str(),
                indent_str)
        }
        Statement::ExitLoop => format!("{}break;", indent_str),
        Statement::Continue => format!("{}continue;", indent_str),
        Statement::Function { name, params, return_type, body } => {
            let return_str = match return_type {
                Some(rt) => format!(" -> {}", transpile_type(rt)),
                None => String::new(),
            };
            let param_str = params.iter()
                .map(|p| format!("{}: {}", p.name, transpile_type(&p.typ)))
                .collect::<Vec<_>>()
                .join(", ");
            let mut output = format!("fn {}({}){}{} {{\n", name, param_str, return_str, indent_str);
            for stmt in body {
                output.push_str(&transpile_statement(stmt, indent + 1));
                output.push('\n');
            }
            output.push_str(&format!("{} }}", indent_str));
            output
        }
        Statement::Return(expr) => {
            match expr {
                Some(e) => format!("{}return {};", indent_str, transpile_expr(e, indent)),
                None => format!("{}return;", indent_str),
            }
        }
        Statement::Expr(e) => {
            format!("{} {};", indent_str, transpile_expr(e, indent))
        }
        Statement::Match { value, arms } => {
            format!("{}match {{:?}} {{\n{}\n{}}}", indent_str, value, indent_str)
        }
        Statement::Try(e) => {
            format!("{}.map_err(|e| e.to_string())?", transpile_expr(e, indent))
        }
        Statement::Cast { expr, typ } => {
            format!("{} as {}", transpile_expr(expr, indent), transpile_type(typ))
        }
        Statement::Clone(e) => format!("{}.clone()", transpile_expr(e, indent)),
        // Statement::Expr(Expression::Vec) removed
    }
}
