// VBR Form Parser and Code Generator
//
// Handles Form...End Form blocks, collects bindings and event handlers,
// and generates idiomatic Rust using vbr_forms_core.
//
// A VBR form file contains:
//   - A Form...End Form block (the UI declaration)
//   - Function blocks (event handler implementations)
//
// The generator produces a complete, compilable .rs file with:
//   - The data struct (one field per Binding)
//   - FormData impl (get/set dispatch by binding name)
//   - Enums for RadioGroup options
//   - The handlers struct + EventDispatch impl
//   - The FormDef builder function
//   - Transpiled event handler functions
//   - main()

// ---------------------------------------------------------------------------
// Parsed control structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ControlKind {
    Label,
    Separator,
    TextBox,
    NumberBox,
    CheckBox,
    RadioGroup,
    DropDown,
    Button,
    Row,
    Group,
    ProgressBar,
    StatusBar,
    Image,
    Svg,
}

#[derive(Debug, Clone)]
pub struct ParsedControl {
    pub kind: ControlKind,
    /// Inline text (Label "...", Button "...")
    pub text: Option<String>,
    /// Style := Bold / Dim / Primary / Danger
    pub style: Option<String>,
    /// OnClick := HandlerName
    pub on_click: Option<String>,
    /// Label := "..."
    pub label: Option<String>,
    /// Binding := variableName
    pub binding: Option<String>,
    /// Placeholder := "..."
    pub placeholder: Option<String>,
    /// MaxLength := n
    pub max_length: Option<u32>,
    /// MultiLine := True
    pub multi_line: bool,
    /// ViewHeight := n
    pub view_height: Option<u32>,
    /// Min := n
    pub min: Option<f64>,
    /// Max := n
    pub max: Option<f64>,
    /// True if min/max have no decimal point → i64 binding
    pub is_int: bool,
    /// Options := "A", "B", "C"
    pub options: Vec<String>,
    /// OnChange := HandlerName
    pub on_change: Option<String>,
    /// Source path (Image, Svg)
    pub source: Option<String>,
    /// Display width in pixels (Image, Svg)
    pub width: Option<f32>,
    /// Display height in pixels (Image, Svg)
    pub height: Option<f32>,
    /// Name := identifier — gives a control a reference-able id for web target
    pub name: Option<String>,
    /// Group title or Row align
    pub extra: Option<String>,
    /// Nested controls (Group, Row)
    pub children: Vec<ParsedControl>,
}

impl ParsedControl {
    fn new(kind: ControlKind) -> Self {
        ParsedControl {
            kind,
            text: None,
            style: None,
            on_click: None,
            label: None,
            binding: None,
            placeholder: None,
            max_length: None,
            multi_line: false,
            view_height: None,
            source: None,
            width: None,
            height: None,
            min: None,
            max: None,
            is_int: true,
            options: Vec::new(),
            on_change: None,
            name: None,
            extra: None,
            children: Vec::new(),
        }
    }

    fn set_property(&mut self, name: &str, raw_value: &str) {
        let value = raw_value.trim();
        let stripped = strip_quotes(value);
        match name {
            "Label"       => self.label = Some(stripped),
            "Text"        => self.text  = Some(stripped),
            "Style"       => self.style    = Some(value.to_string()),
            "OnClick"     => self.on_click = Some(value.to_string()),
            "OnChange"    => self.on_change = Some(value.to_string()),
            "Binding"     => self.binding  = Some(value.to_string()),
            "Placeholder" => self.placeholder = Some(stripped),
            "MaxLength"   => { if let Ok(n) = value.parse::<u32>() { self.max_length = Some(n); } }
            "MultiLine"   => { self.multi_line = value.eq_ignore_ascii_case("true"); }
            "ViewHeight"  => { if let Ok(n) = value.parse::<u32>() { self.view_height = Some(n); } }
            "Source"      => self.source = Some(stripped),
            "Width"       => { if let Ok(n) = value.parse::<f32>() { self.width = Some(n); } }
            "Height"      => { if let Ok(n) = value.parse::<f32>() { self.height = Some(n); } }
            "Min" => {
                if let Ok(n) = value.parse::<f64>() {
                    self.min = Some(n);
                    if value.contains('.') { self.is_int = false; }
                }
            }
            "Max" => {
                if let Ok(n) = value.parse::<f64>() {
                    self.max = Some(n);
                    if value.contains('.') { self.is_int = false; }
                }
            }
            "Options" => self.options = parse_options_list(value),
            "Align"   => self.extra = Some(value.to_string()),
            "Title"   => self.extra = Some(stripped),
            "Name"    => self.name = Some(value.to_string()),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Parsed form and function
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ParsedForm {
    title: String,
    controls: Vec<ParsedControl>,
}

#[derive(Debug)]
struct ParsedFunction {
    name: String,
    body_lines: Vec<String>,
}

// ---------------------------------------------------------------------------
// String utilities
// ---------------------------------------------------------------------------

fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn parse_options_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|part| strip_quotes(part.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// "serverHost" → "server_host"
fn camel_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_lowercase().next().unwrap());
    }
    out
}

/// "connection setup" / "ConnectionSetup" → "ConnectionSetup"
fn to_pascal_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

/// "DarkModeToggled" → "dark_mode_toggled"
fn handler_fn_name(name: &str) -> String {
    camel_to_snake(name)
}

/// Does the handler name suggest "close the form"?
fn is_quit_handler(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("cancel") || lower.contains("close")
        || lower.contains("exit") || lower.contains("quit")
}

// ---------------------------------------------------------------------------
// Block collector — splits input into Form block and Function blocks
// ---------------------------------------------------------------------------

fn collect_blocks(input: &str) -> Result<(Vec<ParsedForm>, Vec<ParsedFunction>), String> {
    let lines: Vec<&str> = input.lines().collect();
    let mut forms: Vec<ParsedForm> = Vec::new();
    let mut functions: Vec<ParsedFunction> = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Form block
        if let Some(rest) = trimmed.strip_prefix("Form ") {
            let title = strip_quotes(rest.trim());
            let mut form_lines: Vec<&str> = Vec::new();
            i += 1;
            let mut depth = 1usize;
            while i < lines.len() {
                let t = lines[i].trim();
                if t.starts_with("Form ") { depth += 1; }
                if t == "End Form" {
                    depth -= 1;
                    if depth == 0 { i += 1; break; }
                }
                form_lines.push(lines[i]);
                i += 1;
            }
            let controls = parse_form_body(&form_lines)?;
            forms.push(ParsedForm { title, controls });
            continue;
        }

        // Function block
        if let Some(rest) = trimmed.strip_prefix("Function ") {
            if let Some(fn_) = parse_function_header(rest) {
                let mut body: Vec<String> = Vec::new();
                i += 1;
                while i < lines.len() {
                    let t = lines[i].trim();
                    if t == "End Function" { i += 1; break; }
                    body.push(lines[i].to_string());
                    i += 1;
                }
                functions.push(ParsedFunction {
                    name: fn_.0,
                    body_lines: body,
                });
                continue;
            }
        }

        i += 1;
    }

    Ok((forms, functions))
}

/// Parse "FnName(param As Type, ...)" → (name, params)
fn parse_function_header(rest: &str) -> Option<(String, Vec<(String, String)>)> {
    let lparen = rest.find('(')?;
    let rparen = rest.rfind(')')?;
    let name = rest[..lparen].trim().to_string();
    let params_str = &rest[lparen + 1..rparen];
    let mut params = Vec::new();
    for param in params_str.split(',') {
        let param = param.trim();
        if param.is_empty() { continue; }
        // "value As String" or "value As Bool"
        let parts: Vec<&str> = param.splitn(3, ' ').collect();
        if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("as") {
            params.push((parts[0].to_string(), parts[2].to_string()));
        } else if !param.is_empty() {
            params.push((param.to_string(), "String".to_string()));
        }
    }
    Some((name, params))
}

// ---------------------------------------------------------------------------
// Form body parser — stack-based control tree builder
// ---------------------------------------------------------------------------

fn parse_form_body(lines: &[&str]) -> Result<Vec<ParsedControl>, String> {
    // Stack frame: (ContainerKind, accumulated_controls)
    // ContainerKind: "group:Title" or "row:Align"
    let mut stack: Vec<(String, Vec<ParsedControl>)> = vec![("root".to_string(), Vec::new())];
    let mut current: Option<ParsedControl> = None;

    for line in lines {
        let trimmed = line.trim();

        // Skip blanks and comments
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        // End Group / End Row
        if trimmed == "End Group" || trimmed == "End Row" {
            // Finalise current control into top of stack
            push_current(&mut stack, &mut current);
            // Pop the frame and turn it into a container control
            let (kind_tag, children) = stack.pop().ok_or("Unexpected End without container")?;
            let container = if kind_tag.starts_with("group:") {
                let mut c = ParsedControl::new(ControlKind::Group);
                c.extra = Some(kind_tag[6..].to_string());
                c.children = children;
                c
            } else {
                // row:Align
                let align = kind_tag.strip_prefix("row:").unwrap_or("").to_string();
                let mut c = ParsedControl::new(ControlKind::Row);
                c.extra = if align.is_empty() { None } else { Some(align) };
                // children should all be Buttons
                c.children = children;
                c
            };
            current = Some(container);
            continue;
        }

        // Property assignment: "PropName := value" — prop name must be a simple identifier
        if let Some(colon_eq) = trimmed.find(":=") {
            let prop_name = trimmed[..colon_eq].trim();
            if !prop_name.contains(' ') {
                let prop_val = trimmed[colon_eq + 2..].trim();
                if let Some(ref mut c) = current {
                    c.set_property(prop_name, prop_val);
                }
                continue;
            }
        }

        // Group "Title"
        if let Some(rest) = trimmed.strip_prefix("Group ") {
            push_current(&mut stack, &mut current);
            let title = strip_quotes(rest.trim());
            stack.push((format!("group:{}", title), Vec::new()));
            continue;
        }

        // Row [Align := ...]  — the Align may be on the same line
        if trimmed.starts_with("Row") {
            push_current(&mut stack, &mut current);
            let align = if let Some(colon_eq) = trimmed.find(":=") {
                trimmed[colon_eq + 2..].trim().to_string()
            } else {
                "Left".to_string()
            };
            stack.push((format!("row:{}", align), Vec::new()));
            continue;
        }

        // Separator
        if trimmed == "Separator" {
            push_current(&mut stack, &mut current);
            current = Some(ParsedControl::new(ControlKind::Separator));
            continue;
        }

        // Control starts with inline text: Label "...", Button "..."
        if let Some(rest) = trimmed.strip_prefix("Label ") {
            push_current(&mut stack, &mut current);
            let mut c = ParsedControl::new(ControlKind::Label);
            c.text = Some(strip_quotes(rest.trim()));
            current = Some(c);
            continue;
        }
        if trimmed == "Label" {
            push_current(&mut stack, &mut current);
            current = Some(ParsedControl::new(ControlKind::Label));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Button ") {
            push_current(&mut stack, &mut current);
            let mut c = ParsedControl::new(ControlKind::Button);
            c.text = Some(strip_quotes(rest.trim()));
            current = Some(c);
            continue;
        }

        // Image / Svg — support inline path or Source property
        for (prefix, kind) in [("Image ", ControlKind::Image), ("Svg ", ControlKind::Svg)] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                push_current(&mut stack, &mut current);
                let mut c = ParsedControl::new(kind);
                c.source = Some(strip_quotes(rest.trim()));
                current = Some(c);
                // handled — break out of inner loop; continue outer
            }
        }
        if matches!(current, Some(ref c) if matches!(c.kind, ControlKind::Image | ControlKind::Svg))
            && trimmed.contains(' ') {
            continue;
        }

        // Bare control keywords
        let kind = match trimmed {
            "TextBox"    => Some(ControlKind::TextBox),
            "NumberBox"  => Some(ControlKind::NumberBox),
            "CheckBox"   => Some(ControlKind::CheckBox),
            "RadioGroup" => Some(ControlKind::RadioGroup),
            "DropDown"   => Some(ControlKind::DropDown),
            "ProgressBar"=> Some(ControlKind::ProgressBar),
            "StatusBar"  => Some(ControlKind::StatusBar),
            "Image"      => Some(ControlKind::Image),
            "Svg"        => Some(ControlKind::Svg),
            _ => None,
        };
        if let Some(k) = kind {
            push_current(&mut stack, &mut current);
            current = Some(ParsedControl::new(k));
        }
        // Any unrecognised line inside a form: ignore
    }

    // Flush final current control
    push_current(&mut stack, &mut current);

    // Return the root frame's controls
    Ok(stack.into_iter().next().map(|(_, c)| c).unwrap_or_default())
}

fn push_current(
    stack: &mut Vec<(String, Vec<ParsedControl>)>,
    current: &mut Option<ParsedControl>,
) {
    if let Some(c) = current.take() {
        if let Some(frame) = stack.last_mut() {
            frame.1.push(c);
        }
    }
}

// ---------------------------------------------------------------------------
// Binding collector — walks the control tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum BindingKind {
    Text,
    NumberInt,
    NumberFloat,
    Bool,
    Progress,
}

#[derive(Debug, Clone)]
struct BindingInfo {
    camel: String,
    snake: String,
    kind: BindingKind,
}

#[derive(Debug, Clone)]
struct HandlerInfo {
    vbr_name: String,    // "HostChanged"
    fn_name: String,     // "host_changed"
    value_kind: Option<BindingKind>,
    is_quit: bool,
}

#[derive(Debug, Clone)]
struct RadioEnumInfo {
    enum_name: String,
    options: Vec<String>,
}

struct Collected {
    bindings: Vec<BindingInfo>,
    handlers: Vec<HandlerInfo>,
    radio_enums: Vec<RadioEnumInfo>,
}

fn collect_metadata(controls: &[ParsedControl]) -> Collected {
    let mut c = Collected {
        bindings: Vec::new(),
        handlers: Vec::new(),
        radio_enums: Vec::new(),
    };
    collect_from_controls(controls, &mut c);
    c
}

fn collect_from_controls(controls: &[ParsedControl], c: &mut Collected) {
    for ctrl in controls {
        collect_from_control(ctrl, c);
    }
}

fn collect_from_control(ctrl: &ParsedControl, c: &mut Collected) {
    match ctrl.kind {
        ControlKind::TextBox | ControlKind::DropDown => {
            if let Some(ref b) = ctrl.binding {
                add_binding(c, b, BindingKind::Text);
            }
            if let Some(ref h) = ctrl.on_change {
                add_handler(c, h, Some(BindingKind::Text));
            }
        }
        ControlKind::NumberBox => {
            if let Some(ref b) = ctrl.binding {
                let kind = if ctrl.is_int { BindingKind::NumberInt } else { BindingKind::NumberFloat };
                add_binding(c, b, kind.clone());
                if let Some(ref h) = ctrl.on_change {
                    add_handler(c, h, Some(kind));
                }
            }
        }
        ControlKind::CheckBox => {
            if let Some(ref b) = ctrl.binding {
                add_binding(c, b, BindingKind::Bool);
            }
            if let Some(ref h) = ctrl.on_change {
                add_handler(c, h, Some(BindingKind::Bool));
            }
        }
        ControlKind::RadioGroup => {
            if let Some(ref b) = ctrl.binding {
                add_binding(c, b, BindingKind::Text);
                // Generate enum from options + binding name
                if !ctrl.options.is_empty() {
                    let enum_name = {
                        let snake = camel_to_snake(b);
                        to_pascal_case(&snake.replace('_', " "))
                    };
                    if c.radio_enums.iter().all(|e| e.enum_name != enum_name) {
                        c.radio_enums.push(RadioEnumInfo {
                            enum_name,
                            options: ctrl.options.clone(),
                        });
                    }
                }
            }
            if let Some(ref h) = ctrl.on_change {
                add_handler(c, h, Some(BindingKind::Text));
            }
        }
        ControlKind::ProgressBar => {
            if let Some(ref b) = ctrl.binding {
                add_binding(c, b, BindingKind::Progress);
            }
        }
        ControlKind::StatusBar => {
            if let Some(ref b) = ctrl.binding {
                add_binding(c, b, BindingKind::Text);
            }
        }
        ControlKind::Button => {
            if let Some(ref h) = ctrl.on_click {
                add_handler(c, h, None);
            }
        }
        ControlKind::Label => {
            if let Some(ref h) = ctrl.on_click {
                add_handler(c, h, None);
            }
        }
        ControlKind::Group | ControlKind::Row => {
            collect_from_controls(&ctrl.children, c);
        }
        _ => {}
    }
}

fn add_binding(c: &mut Collected, camel: &str, kind: BindingKind) {
    if c.bindings.iter().all(|b| b.camel != camel) {
        c.bindings.push(BindingInfo {
            camel: camel.to_string(),
            snake: camel_to_snake(camel),
            kind,
        });
    }
}

fn add_handler(c: &mut Collected, name: &str, value_kind: Option<BindingKind>) {
    if c.handlers.iter().all(|h| h.vbr_name != name) {
        c.handlers.push(HandlerInfo {
            vbr_name: name.to_string(),
            fn_name: handler_fn_name(name),
            value_kind,
            is_quit: is_quit_handler(name),
        });
    }
}

// ---------------------------------------------------------------------------
// Code generator
// ---------------------------------------------------------------------------

pub fn transpile_form_file(input: &str, target: &str) -> Result<String, String> {
    let (forms, functions) = collect_blocks(input)?;
    if forms.is_empty() {
        return Err("No Form block found".to_string());
    }
    let form = &forms[0];
    let meta = collect_metadata(&form.controls);
    let struct_name = to_pascal_case(&form.title);

    let mut out = String::new();

    // --- Preamble ---
    out.push_str("// Generated by VBR transpiler\n");
    out.push_str("// Edit the handler function bodies below.\n\n");
    out.push_str("use std::sync::{Arc, Mutex};\n");
    out.push_str("use vbr_forms_core::*;\n");
    let (backend_use, backend_name) = match target {
        "egui" => ("use vbr_forms_egui::EguiBackend;\n\n", "EguiBackend"),
        _      => ("use vbr_forms_cursive::CursiveBackend;\n\n", "CursiveBackend"),
    };
    out.push_str(backend_use);

    // --- Data struct ---
    out.push_str(&format!("// Generated from Form \"{}\"\n", form.title));
    out.push_str("#[derive(Debug, Default)]\n");
    out.push_str(&format!("struct {}Data {{\n", struct_name));
    for b in &meta.bindings {
        let rust_type = match b.kind {
            BindingKind::Text => "String",
            BindingKind::NumberInt => "i64",
            BindingKind::NumberFloat => "f64",
            BindingKind::Bool => "bool",
            BindingKind::Progress => "f32",
        };
        out.push_str(&format!("    pub {}: {},\n", b.snake, rust_type));
    }
    out.push_str("}\n\n");

    // --- FormData impl ---
    out.push_str(&format!("impl FormData for {}Data {{\n", struct_name));
    out.push_str("    fn get(&self, binding: &str) -> Option<FieldValue> {\n");
    out.push_str("        match binding {\n");
    for b in &meta.bindings {
        let expr = match b.kind {
            BindingKind::Text => format!("FieldValue::Text(self.{}.clone())", b.snake),
            BindingKind::NumberInt => format!("FieldValue::Number(self.{} as f64)", b.snake),
            BindingKind::NumberFloat => format!("FieldValue::Number(self.{})", b.snake),
            BindingKind::Bool => format!("FieldValue::Bool(self.{})", b.snake),
            BindingKind::Progress => format!("FieldValue::Progress(self.{})", b.snake),
        };
        out.push_str(&format!("            \"{}\" => Some({}),\n", b.camel, expr));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    fn set(&mut self, binding: &str, value: FieldValue) {\n");
    out.push_str("        match (binding, value) {\n");
    for b in &meta.bindings {
        let pat = match b.kind {
            BindingKind::Text => "FieldValue::Text(v)",
            BindingKind::NumberInt => "FieldValue::Number(v)",
            BindingKind::NumberFloat => "FieldValue::Number(v)",
            BindingKind::Bool => "FieldValue::Bool(v)",
            BindingKind::Progress => "FieldValue::Progress(v)",
        };
        let assignment = match b.kind {
            BindingKind::NumberInt => format!("self.{} = v as i64", b.snake),
            _ => format!("self.{} = v", b.snake),
        };
        out.push_str(&format!(
            "            (\"{}\", {}) => {},\n",
            b.camel, pat, assignment
        ));
    }
    out.push_str("            _ => {}\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // --- Enums for RadioGroup ---
    for re in &meta.radio_enums {
        out.push_str("#[derive(Debug, Clone, Default)]\n");
        out.push_str(&format!("pub enum {} {{\n", re.enum_name));
        for (i, opt) in re.options.iter().enumerate() {
            let variant = to_pascal_case(opt);
            if i == 0 {
                out.push_str(&format!("    #[default]\n    {},\n", variant));
            } else {
                out.push_str(&format!("    {},\n", variant));
            }
        }
        out.push_str("}\n\n");
    }

    // --- Handlers struct ---
    out.push_str(&format!("struct {}Handlers {{\n", struct_name));
    out.push_str(&format!("    data: Arc<Mutex<{}Data>>,\n", struct_name));
    out.push_str("}\n\n");

    // --- EventDispatch impl ---
    out.push_str(&format!("impl EventDispatch for {}Handlers {{\n", struct_name));
    out.push_str("    fn dispatch(&mut self, handler: &str, value: FieldValue) -> Action {\n");
    out.push_str("        match handler {\n");
    for h in &meta.handlers {
        let call = if h.value_kind.is_some() {
            format!("{}(&mut self.data.lock().unwrap(), value)", h.fn_name)
        } else {
            format!("{}(&mut self.data.lock().unwrap())", h.fn_name)
        };
        let action = if h.is_quit { "Action::Quit" } else { "Action::None" };
        out.push_str(&format!(
            "            \"{}\" => {{ {}; {} }}\n",
            h.vbr_name, call, action
        ));
    }
    out.push_str("            _ => Action::None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // --- FormDef builder ---
    let builder_name = format!("build_{}_form", camel_to_snake(&struct_name));
    out.push_str(&format!("fn {}() -> FormDef {{\n", builder_name));
    out.push_str(&format!("    FormDef::new(\"{}\")\n", form.title));
    for ctrl in &form.controls {
        emit_control(&mut out, ctrl, 2);
    }
    // Remove trailing newline and close
    if out.ends_with('\n') { out.pop(); }
    out.push_str("\n}\n\n");

    // --- Handler function implementations ---
    // Collect VBR-declared functions indexed by name
    let fn_map: std::collections::HashMap<&str, &ParsedFunction> =
        functions.iter().map(|f| (f.name.as_str(), f)).collect();

    for h in &meta.handlers {
        let param_sig = match &h.value_kind {
            Some(BindingKind::Text)        => "value: FieldValue",
            Some(BindingKind::Bool)        => "value: FieldValue",
            Some(BindingKind::NumberInt)   => "value: FieldValue",
            Some(BindingKind::NumberFloat) => "value: FieldValue",
            Some(BindingKind::Progress)    => "value: FieldValue",
            None                           => "",
        };

        out.push_str(&format!(
            "fn {}(data: &mut {}Data{}{}) {{\n",
            h.fn_name,
            struct_name,
            if param_sig.is_empty() { "" } else { ", " },
            param_sig
        ));

        // Unwrap FieldValue to the native type so body code can use `value` directly
        if let Some(ref vk) = h.value_kind {
            let extraction = match vk {
                BindingKind::Text        => "    let value = match value { FieldValue::Text(s) => s, _ => String::new() };\n",
                BindingKind::Bool        => "    let value = match value { FieldValue::Bool(b) => b, _ => false };\n",
                BindingKind::NumberInt   => "    let value = match value { FieldValue::Number(n) => n as i64, _ => 0 };\n",
                BindingKind::NumberFloat => "    let value = match value { FieldValue::Number(n) => n, _ => 0.0 };\n",
                BindingKind::Progress    => "    let value = match value { FieldValue::Progress(p) => p, _ => 0.0_f32 };\n",
            };
            out.push_str(extraction);
        }

        if let Some(vbr_fn) = fn_map.get(h.vbr_name.as_str()) {
            let body = transpile_fn_body(&vbr_fn.body_lines, &meta.bindings, h.value_kind.as_ref());
            if body.is_empty() {
                out.push_str("    // TODO: implement\n");
            } else {
                out.push_str(&body);
            }
        } else {
            out.push_str("    // TODO: implement\n");
        }

        out.push_str("}\n\n");
    }

    // --- main() ---
    out.push_str("fn main() {\n");
    out.push_str(&format!(
        "    let data = Arc::new(Mutex::new({}Data::default()));\n",
        struct_name
    ));
    out.push_str(&format!(
        "    let handlers = Arc::new(Mutex::new({}Handlers {{ data: Arc::clone(&data) }}));\n",
        struct_name
    ));
    out.push_str(&format!(
        "    {}::run(\n        {}(),\n        data as Arc<Mutex<dyn FormData>>,\n        handlers as Arc<Mutex<dyn EventDispatch>>,\n    ).unwrap();\n",
        backend_name, builder_name
    ));
    out.push_str("}\n");

    Ok(out)
}

// ---------------------------------------------------------------------------
// FormDef builder code emission
// ---------------------------------------------------------------------------

fn emit_control(out: &mut String, ctrl: &ParsedControl, depth: usize) {
    let pad = "    ".repeat(depth);
    let inner_pad = "    ".repeat(depth + 1);

    match ctrl.kind {
        ControlKind::Label => {
            let text = ctrl.text.as_deref()
                .or(ctrl.label.as_deref())
                .unwrap_or("");
            let style_suffix = match ctrl.style.as_deref() {
                Some("Bold") => ".style(LabelStyle::Bold)".to_string(),
                Some("Dim")  => ".style(LabelStyle::Dim)".to_string(),
                _ => String::new(),
            };
            let click_suffix = match &ctrl.on_click {
                Some(h) => format!(".on_click(\"{}\")", h),
                None => String::new(),
            };
            out.push_str(&format!(
                "{}.add(Control::Label(LabelDef::new(\"{}\"){}{}))\n",
                pad, text, style_suffix, click_suffix
            ));
        }

        ControlKind::Separator => {
            out.push_str(&format!("{}.add(Control::Separator)\n", pad));
        }

        ControlKind::TextBox => {
            let label = ctrl.label.as_deref().unwrap_or("");
            let binding = ctrl.binding.as_deref().unwrap_or("");
            let mut suffix = String::new();
            if let Some(ref p) = ctrl.placeholder {
                suffix.push_str(&format!(".placeholder(\"{}\")", p));
            }
            if let Some(n) = ctrl.max_length {
                suffix.push_str(&format!(".max_length({})", n));
            }
            if ctrl.multi_line {
                suffix.push_str(".multi_line(true)");
            }
            if let Some(n) = ctrl.view_height {
                suffix.push_str(&format!(".view_height({})", n));
            }
            if let Some(ref h) = ctrl.on_change {
                if !ctrl.multi_line {
                    suffix.push_str(&format!(".on_change(\"{}\")", h));
                }
            }
            out.push_str(&format!(
                "{}.add(Control::TextBox(TextBoxDef::new(\"{}\", \"{}\"){}))\n",
                pad, label, binding, suffix
            ));
        }

        ControlKind::NumberBox => {
            let label = ctrl.label.as_deref().unwrap_or("");
            let binding = ctrl.binding.as_deref().unwrap_or("");
            let mut suffix = String::new();
            if let Some(n) = ctrl.min { suffix.push_str(&format!(".min({}_f64)", n)); }
            if let Some(n) = ctrl.max { suffix.push_str(&format!(".max({}_f64)", n)); }
            if let Some(ref h) = ctrl.on_change { suffix.push_str(&format!(".on_change(\"{}\")", h)); }
            out.push_str(&format!(
                "{}.add(Control::NumberBox(NumberBoxDef::new(\"{}\", \"{}\"){}))\n",
                pad, label, binding, suffix
            ));
        }

        ControlKind::CheckBox => {
            let label = ctrl.label.as_deref().unwrap_or("");
            let binding = ctrl.binding.as_deref().unwrap_or("");
            let suffix = ctrl.on_change.as_ref()
                .map(|h| format!(".on_change(\"{}\")", h))
                .unwrap_or_default();
            out.push_str(&format!(
                "{}.add(Control::CheckBox(CheckBoxDef::new(\"{}\", \"{}\"){}))\n",
                pad, label, binding, suffix
            ));
        }

        ControlKind::RadioGroup => {
            let label = ctrl.label.as_deref().unwrap_or("");
            let binding = ctrl.binding.as_deref().unwrap_or("");
            let opts = ctrl.options.iter()
                .map(|o| format!("\"{}\"", o))
                .collect::<Vec<_>>()
                .join(", ");
            let suffix = ctrl.on_change.as_ref()
                .map(|h| format!(".on_change(\"{}\")", h))
                .unwrap_or_default();
            out.push_str(&format!(
                "{}.add(Control::RadioGroup(\n{}RadioGroupDef::new(\"{}\", \"{}\")\n{}.options(vec![{}]){}\n{}))\n",
                pad, inner_pad, label, binding, inner_pad, opts, suffix, pad
            ));
        }

        ControlKind::DropDown => {
            let label = ctrl.label.as_deref().unwrap_or("");
            let binding = ctrl.binding.as_deref().unwrap_or("");
            let opts = ctrl.options.iter()
                .map(|o| format!("\"{}\"", o))
                .collect::<Vec<_>>()
                .join(", ");
            let suffix = ctrl.on_change.as_ref()
                .map(|h| format!(".on_change(\"{}\")", h))
                .unwrap_or_default();
            out.push_str(&format!(
                "{}.add(Control::DropDown(\n{}DropDownDef::new(\"{}\", \"{}\")\n{}.options(vec![{}]){}\n{}))\n",
                pad, inner_pad, label, binding, inner_pad, opts, suffix, pad
            ));
        }

        ControlKind::Button => {
            let text = ctrl.text.as_deref().unwrap_or("");
            let handler = ctrl.on_click.as_deref().unwrap_or("");
            let style_suffix = match ctrl.style.as_deref() {
                Some("Primary") => ".style(ButtonStyle::Primary)".to_string(),
                Some("Danger")  => ".style(ButtonStyle::Danger)".to_string(),
                _ => String::new(),
            };
            out.push_str(&format!(
                "{}.add(ButtonDef::new(\"{}\", \"{}\"){})\n",
                pad, text, handler, style_suffix
            ));
        }

        ControlKind::Row => {
            let align = match ctrl.extra.as_deref() {
                Some("SpaceBetween") => "RowAlign::SpaceBetween",
                Some("Center")       => "RowAlign::Center",
                Some("Right")        => "RowAlign::Right",
                _                    => "RowAlign::Left",
            };
            out.push_str(&format!("{}.add(Control::Row(\n{}RowDef::new({})\n", pad, inner_pad, align));
            for child in &ctrl.children {
                emit_control(out, child, depth + 1);
            }
            out.push_str(&format!("{}))\n", pad));
        }

        ControlKind::Group => {
            let title = ctrl.extra.as_deref().unwrap_or("");
            out.push_str(&format!("{}.add(Control::Group(\n{}GroupDef::new(\"{}\")\n", pad, inner_pad, title));
            for child in &ctrl.children {
                emit_control(out, child, depth + 1);
            }
            out.push_str(&format!("{}))\n", pad));
        }

        ControlKind::ProgressBar => {
            let label = ctrl.label.as_deref().unwrap_or("");
            let binding = ctrl.binding.as_deref().unwrap_or("");
            out.push_str(&format!(
                "{}.add(Control::ProgressBar(ProgressBarDef::new(\"{}\", \"{}\")))\n",
                pad, label, binding
            ));
        }

        ControlKind::StatusBar => {
            let binding = ctrl.binding.as_deref().unwrap_or("");
            out.push_str(&format!(
                "{}.add(Control::StatusBar(StatusBarDef::new(\"{}\")))\n",
                pad, binding
            ));
        }

        ControlKind::Image => {
            let source = ctrl.source.as_deref().unwrap_or("");
            let mut suffix = String::new();
            if let Some(w) = ctrl.width  { suffix.push_str(&format!(".width({}_f32)", w)); }
            if let Some(h) = ctrl.height { suffix.push_str(&format!(".height({}_f32)", h)); }
            out.push_str(&format!(
                "{}.add(Control::Image(ImageDef::new(\"{}\"){}))\n",
                pad, source, suffix
            ));
        }

        ControlKind::Svg => {
            let source = ctrl.source.as_deref().unwrap_or("");
            let mut suffix = String::new();
            if let Some(w) = ctrl.width  { suffix.push_str(&format!(".width({}_f32)", w)); }
            if let Some(h) = ctrl.height { suffix.push_str(&format!(".height({}_f32)", h)); }
            out.push_str(&format!(
                "{}.add(Control::Svg(SvgDef::new(\"{}\"){}))\n",
                pad, source, suffix
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Function body transpiler — simple, binding-aware
// ---------------------------------------------------------------------------

fn transpile_fn_body(lines: &[String], bindings: &[BindingInfo], value_kind: Option<&BindingKind>) -> String {
    let mut out = String::new();
    let mut if_depth = 0usize;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        let indent = "    ".repeat(1 + if_depth);

        // End If
        if trimmed.eq_ignore_ascii_case("End If") {
            if if_depth > 0 { if_depth -= 1; }
            let close_indent = "    ".repeat(1 + if_depth);
            out.push_str(&format!("{}}}\n", close_indent));
            continue;
        }

        // Else
        if trimmed.eq_ignore_ascii_case("Else") {
            if if_depth > 0 { if_depth -= 1; }
            let close_indent = "    ".repeat(1 + if_depth);
            out.push_str(&format!("{}}} else {{\n", close_indent));
            if_depth += 1;
            continue;
        }

        // If ... Then
        if trimmed.to_lowercase().starts_with("if ") {
            if let Some(cond) = extract_if_condition(trimmed) {
                let cond_rust = bool_coerce_expr(&cond, bindings, value_kind);
                out.push_str(&format!("{}if {} {{\n", indent, cond_rust));
                if_depth += 1;
                continue;
            }
        }

        // Exit Function
        if trimmed.eq_ignore_ascii_case("Exit Function") {
            out.push_str(&format!("{}return;\n", indent));
            continue;
        }

        // Assignment: binding = value  OR  binding = string & other
        if let Some((lhs, rhs)) = split_assignment(trimmed) {
            if let Some(binding) = bindings.iter().find(|b| b.camel == lhs) {
                let rhs_rust = transpile_rhs(&rhs, binding, bindings);
                out.push_str(&format!("{}data.{} = {};\n", indent, binding.snake, rhs_rust));
                continue;
            }
        }

        // Fallback: emit as comment
        out.push_str(&format!("{}// {}\n", indent, trimmed));
    }

    out
}

/// Coerce a VBR boolean-context expression to valid Rust.
/// VBA treats non-empty strings and non-zero numbers as truthy.
fn bool_coerce_expr(expr: &str, bindings: &[BindingInfo], value_kind: Option<&BindingKind>) -> String {
    let e = expr.trim();
    // Single bare identifier (no operators, parens, spaces)
    if !e.is_empty() && e.chars().all(|c| c.is_alphanumeric() || c == '_') {
        if e == "value" {
            return match value_kind {
                Some(BindingKind::Text)    => "!value.is_empty()".to_string(),
                Some(BindingKind::NumberInt | BindingKind::NumberFloat) => "value != 0".to_string(),
                _ => e.to_string(),
            };
        }
        if let Some(b) = bindings.iter().find(|b| b.camel == e || b.snake == e) {
            return match b.kind {
                BindingKind::Text        => format!("!data.{}.is_empty()", b.snake),
                BindingKind::NumberInt
                | BindingKind::NumberFloat => format!("data.{} != 0", b.snake),
                _ => format!("data.{}", b.snake),
            };
        }
    }
    transpile_expr_simple(expr)
}

fn extract_if_condition(line: &str) -> Option<String> {
    let after_if = line.strip_prefix("If ")
        .or_else(|| line.strip_prefix("if "))?;
    let cond = if let Some(pos) = after_if.to_lowercase().rfind(" then") {
        after_if[..pos].trim().to_string()
    } else {
        after_if.trim().to_string()
    };
    Some(cond)
}

fn split_assignment(line: &str) -> Option<(String, String)> {
    // Simple `lhs = rhs`, but skip `==` comparisons
    let eq_pos = line.find('=')?;
    if line.as_bytes().get(eq_pos.saturating_sub(1)) == Some(&b'!') { return None; }
    if line.as_bytes().get(eq_pos + 1) == Some(&b'=') { return None; }
    let lhs = line[..eq_pos].trim().to_string();
    let rhs = line[eq_pos + 1..].trim().to_string();
    if lhs.contains(' ') || lhs.contains('(') { return None; }
    Some((lhs, rhs))
}

fn transpile_rhs(rhs: &str, binding: &BindingInfo, bindings: &[BindingInfo]) -> String {
    let rhs = rhs.trim();
    // String concatenation with &
    if rhs.contains(" & ") {
        let parts: Vec<&str> = rhs.split(" & ").collect();
        let fmt = "{}".repeat(parts.len());
        let args: String = parts.iter()
            .map(|p| {
                let p = p.trim();
                // Substitute binding camelCase names with data.snake_name
                if let Some(b) = bindings.iter().find(|b| b.camel == p) {
                    format!(", data.{}", b.snake)
                } else {
                    format!(", {}", p)
                }
            })
            .collect();
        return format!("format!(\"{}\"{})", fmt, args);
    }

    // String literal → String::from
    if rhs.starts_with('"') && rhs.ends_with('"') {
        return format!("String::from({})", rhs);
    }

    // Matches a binding name → data.field
    match binding.kind {
        BindingKind::Text     => format!("{}.to_string()", rhs),
        BindingKind::NumberInt   => rhs.to_string(),
        BindingKind::NumberFloat => rhs.to_string(),
        BindingKind::Bool     => rhs.to_string(),
        BindingKind::Progress => rhs.to_string(),
    }
}

fn transpile_expr_simple(expr: &str) -> String {
    // Very basic: replace VBR comparisons and known functions
    expr
        .replace("<>", "!=")
        .replace("And ", "&& ")
        .replace("Or ", "|| ")
        .replace("Not ", "!")
        .replace("True", "true")
        .replace("False", "false")
}

// ===========================================================================
// Web / WASM transpiler
// ===========================================================================

// ---------------------------------------------------------------------------
// Named control info — Label or Button controls with Name := id
// ---------------------------------------------------------------------------

struct NamedCtrl {
    /// snake_case identifier used in struct fields and snapshot
    id: String,
    /// Original camelCase name from Name := property
    vbr_name: String,
    /// Initial caption text
    initial_caption: String,
}

fn collect_named_ctrls(controls: &[ParsedControl]) -> Vec<NamedCtrl> {
    let mut out = Vec::new();
    collect_named_recursive(controls, &mut out);
    out
}

fn collect_named_recursive(controls: &[ParsedControl], out: &mut Vec<NamedCtrl>) {
    for ctrl in controls {
        if let Some(ref name) = ctrl.name {
            match ctrl.kind {
                ControlKind::Label | ControlKind::Button => {
                    let initial = ctrl.text.as_deref()
                        .or(ctrl.label.as_deref())
                        .unwrap_or("")
                        .to_string();
                    out.push(NamedCtrl {
                        id: camel_to_snake(name),
                        vbr_name: name.clone(),
                        initial_caption: initial,
                    });
                }
                _ => {}
            }
        }
        match ctrl.kind {
            ControlKind::Group | ControlKind::Row => {
                collect_named_recursive(&ctrl.children, out);
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Generate (wasm_rust_source, json_layout) for `--target web`.
pub fn transpile_web_form_file(input: &str) -> Result<(String, String), String> {
    let (forms, functions) = collect_blocks(input)?;
    if forms.is_empty() {
        return Err("No Form block found".to_string());
    }
    let form = &forms[0];
    let meta = collect_metadata(&form.controls);
    let named = collect_named_ctrls(&form.controls);
    let struct_name = to_pascal_case(&form.title);

    let rust = emit_wasm_rust(form, &functions, &meta, &named, &struct_name);
    let json = emit_layout_json(form);
    Ok((rust, json))
}

// ---------------------------------------------------------------------------
// WASM Rust emitter
// ---------------------------------------------------------------------------

fn emit_wasm_rust(
    _form: &ParsedForm,
    functions: &[ParsedFunction],
    meta: &Collected,
    named: &[NamedCtrl],
    struct_name: &str,
) -> String {
    let mut out = String::new();

    out.push_str("// Generated by VBR transpiler — web/WASM target\n");
    out.push_str("//\n");
    out.push_str("// Build with: wasm-pack build --target bundler\n");
    out.push_str("//\n");
    out.push_str("// Required Cargo.toml additions:\n");
    out.push_str("//   [lib]\n");
    out.push_str("//   crate-type = [\"cdylib\"]\n");
    out.push_str("//   [dependencies]\n");
    out.push_str("//   vbr_forms_web    = { path = \"../vbr_forms_web\" }\n");
    out.push_str("//   wasm-bindgen     = \"0.2\"\n");
    out.push_str("//   serde-wasm-bindgen = \"0.6\"\n\n");

    out.push_str("use wasm_bindgen::prelude::*;\n");
    out.push_str("use vbr_forms_web::{FormSnapshot, ControlState};\n\n");

    // --- Struct definition ---
    out.push_str("#[wasm_bindgen]\n");
    out.push_str(&format!("pub struct {} {{\n", struct_name));
    for b in &meta.bindings {
        let rust_type = match b.kind {
            BindingKind::Text        => "String",
            BindingKind::NumberInt   => "i64",
            BindingKind::NumberFloat => "f64",
            BindingKind::Bool        => "bool",
            BindingKind::Progress    => "f32",
        };
        out.push_str(&format!("    {}: {},\n", b.snake, rust_type));
    }
    for nc in named {
        out.push_str(&format!("    {}_visible: bool,\n", nc.id));
        out.push_str(&format!("    {}_caption: String,\n", nc.id));
    }
    out.push_str("}\n\n");

    // --- impl block ---
    out.push_str("#[wasm_bindgen]\n");
    out.push_str(&format!("impl {} {{\n", struct_name));

    // constructor
    out.push_str("    #[wasm_bindgen(constructor)]\n");
    out.push_str("    pub fn new() -> Self {\n");
    out.push_str(&format!("        {} {{\n", struct_name));
    for b in &meta.bindings {
        let init = match b.kind {
            BindingKind::Text        => "String::new()".to_string(),
            BindingKind::NumberInt   => "0".to_string(),
            BindingKind::NumberFloat => "0.0".to_string(),
            BindingKind::Bool        => "false".to_string(),
            BindingKind::Progress    => "0.0".to_string(),
        };
        out.push_str(&format!("            {}: {},\n", b.snake, init));
    }
    for nc in named {
        out.push_str(&format!("            {}_visible: true,\n", nc.id));
        out.push_str(&format!("            {}_caption: \"{}\".to_string(),\n", nc.id, escape_json_str(&nc.initial_caption)));
    }
    out.push_str("        }\n    }\n\n");

    // initial_snapshot
    out.push_str("    pub fn initial_snapshot(&self) -> JsValue {\n");
    out.push_str("        serde_wasm_bindgen::to_value(&self.snapshot()).unwrap()\n");
    out.push_str("    }\n\n");

    // event handler methods
    let fn_map: std::collections::HashMap<&str, &ParsedFunction> =
        functions.iter().map(|f| (f.name.as_str(), f)).collect();

    for h in &meta.handlers {
        let param = match &h.value_kind {
            Some(BindingKind::Text)                    => ", value: String",
            Some(BindingKind::NumberInt)               => ", value: f64",
            Some(BindingKind::NumberFloat)             => ", value: f64",
            Some(BindingKind::Bool)                    => ", value: bool",
            Some(BindingKind::Progress)                => ", value: f32",
            None                                       => "",
        };
        out.push_str(&format!(
            "    pub fn {}(&mut self{}) -> JsValue {{\n",
            h.fn_name, param
        ));
        if let Some(vbr_fn) = fn_map.get(h.vbr_name.as_str()) {
            let body = transpile_web_fn_body(
                &vbr_fn.body_lines,
                &meta.bindings,
                named,
                h.value_kind.as_ref(),
            );
            out.push_str(&body);
        }
        out.push_str("        serde_wasm_bindgen::to_value(&self.snapshot()).unwrap()\n");
        out.push_str("    }\n\n");
    }

    // private snapshot()
    out.push_str("    fn snapshot(&self) -> FormSnapshot {\n");
    out.push_str("        FormSnapshot {\n");
    out.push_str("            controls: vec![\n");
    for b in &meta.bindings {
        let val = match b.kind {
            BindingKind::Text        => format!("self.{}.clone()", b.snake),
            BindingKind::NumberInt   => format!("self.{}.to_string()", b.snake),
            BindingKind::NumberFloat => format!("self.{}.to_string()", b.snake),
            BindingKind::Bool        => format!("self.{}.to_string()", b.snake),
            BindingKind::Progress    => format!("self.{}.to_string()", b.snake),
        };
        out.push_str(&format!(
            "                ControlState {{ id: \"{id}\".to_string(), value: {val}, caption: None, visible: true, enabled: true, error: None }},\n",
            id = b.snake, val = val
        ));
    }
    for nc in named {
        out.push_str(&format!(
            "                ControlState {{ id: \"{id}\".to_string(), value: String::new(), caption: Some(self.{id}_caption.clone()), visible: self.{id}_visible, enabled: true, error: None }},\n",
            id = nc.id
        ));
    }
    out.push_str("            ],\n        }\n    }\n}\n");

    out
}

// ---------------------------------------------------------------------------
// JSON layout emitter
// ---------------------------------------------------------------------------

fn escape_json_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn emit_layout_json(form: &ParsedForm) -> String {
    let mut out = String::new();
    let mut counter = 0usize;
    out.push_str("{\n");
    out.push_str(&format!("  \"title\": \"{}\",\n", escape_json_str(&form.title)));
    out.push_str("  \"controls\": [\n");
    let n = form.controls.len();
    for (i, ctrl) in form.controls.iter().enumerate() {
        emit_layout_ctrl_json(&mut out, ctrl, 4, i == n - 1, &mut counter);
    }
    out.push_str("  ]\n}\n");
    out
}

fn emit_layout_ctrl_json(
    out: &mut String,
    ctrl: &ParsedControl,
    indent: usize,
    is_last: bool,
    counter: &mut usize,
) {
    let pad = " ".repeat(indent);
    let end = if is_last { "" } else { "," };

    match ctrl.kind {
        ControlKind::Separator => {
            out.push_str(&format!("{}{{ \"type\": \"Separator\" }}{}\n", pad, end));
        }

        ControlKind::Label => {
            let text = ctrl.text.as_deref().or(ctrl.label.as_deref()).unwrap_or("");
            let id = ctrl.name.as_ref()
                .map(|n| camel_to_snake(n))
                .unwrap_or_else(|| { *counter += 1; format!("label_{}", counter) });
            let mut f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"Label\""),
                format!("\"text\": \"{}\"", escape_json_str(text)),
            ];
            if let Some(ref s) = ctrl.style {
                f.push(format!("\"style\": \"{}\"", s));
            }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::TextBox => {
            let id = ctrl.binding.as_deref().map(camel_to_snake).unwrap_or_default();
            let mut f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"TextBox\""),
                format!("\"label\": \"{}\"", escape_json_str(ctrl.label.as_deref().unwrap_or(""))),
            ];
            if ctrl.multi_line { f.push("\"multiLine\": true".to_string()); }
            if let Some(n) = ctrl.view_height { f.push(format!("\"viewHeight\": {}", n)); }
            if let Some(ref p) = ctrl.placeholder { f.push(format!("\"placeholder\": \"{}\"", escape_json_str(p))); }
            if let Some(ref h) = ctrl.on_change {
                f.push(format!("\"events\": {{ \"onChange\": \"{}\" }}", handler_fn_name(h)));
            }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::NumberBox => {
            let id = ctrl.binding.as_deref().map(camel_to_snake).unwrap_or_default();
            let mut f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"NumberBox\""),
                format!("\"label\": \"{}\"", escape_json_str(ctrl.label.as_deref().unwrap_or(""))),
            ];
            if let Some(v) = ctrl.min { f.push(format!("\"min\": {}", v)); }
            if let Some(v) = ctrl.max { f.push(format!("\"max\": {}", v)); }
            if let Some(ref h) = ctrl.on_change {
                f.push(format!("\"events\": {{ \"onChange\": \"{}\" }}", handler_fn_name(h)));
            }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::CheckBox => {
            let id = ctrl.binding.as_deref().map(camel_to_snake).unwrap_or_default();
            let mut f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"CheckBox\""),
                format!("\"label\": \"{}\"", escape_json_str(ctrl.label.as_deref().unwrap_or(""))),
            ];
            if let Some(ref h) = ctrl.on_change {
                f.push(format!("\"events\": {{ \"onChange\": \"{}\" }}", handler_fn_name(h)));
            }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::RadioGroup => {
            let id = ctrl.binding.as_deref().map(camel_to_snake).unwrap_or_default();
            let opts = ctrl.options.iter()
                .map(|o| format!("\"{}\"", escape_json_str(o)))
                .collect::<Vec<_>>()
                .join(", ");
            let mut f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"RadioGroup\""),
                format!("\"label\": \"{}\"", escape_json_str(ctrl.label.as_deref().unwrap_or(""))),
                format!("\"options\": [{}]", opts),
            ];
            if let Some(ref h) = ctrl.on_change {
                f.push(format!("\"events\": {{ \"onChange\": \"{}\" }}", handler_fn_name(h)));
            }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::DropDown => {
            let id = ctrl.binding.as_deref().map(camel_to_snake).unwrap_or_default();
            let opts = ctrl.options.iter()
                .map(|o| format!("\"{}\"", escape_json_str(o)))
                .collect::<Vec<_>>()
                .join(", ");
            let mut f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"DropDown\""),
                format!("\"label\": \"{}\"", escape_json_str(ctrl.label.as_deref().unwrap_or(""))),
                format!("\"options\": [{}]", opts),
            ];
            if let Some(ref h) = ctrl.on_change {
                f.push(format!("\"events\": {{ \"onChange\": \"{}\" }}", handler_fn_name(h)));
            }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::Button => {
            let id = ctrl.name.as_ref()
                .map(|n| camel_to_snake(n))
                .or_else(|| ctrl.on_click.as_ref().map(|h| handler_fn_name(h)))
                .unwrap_or_default();
            let text = ctrl.text.as_deref().or(ctrl.label.as_deref()).unwrap_or("");
            let mut f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"Button\""),
                format!("\"text\": \"{}\"", escape_json_str(text)),
            ];
            if let Some(ref s) = ctrl.style {
                f.push(format!("\"style\": \"{}\"", s));
            }
            if let Some(ref h) = ctrl.on_click {
                f.push(format!("\"events\": {{ \"onClick\": \"{}\" }}", handler_fn_name(h)));
            }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::Row => {
            let align = ctrl.extra.as_deref().unwrap_or("Left");
            out.push_str(&format!("{}{{\n", pad));
            out.push_str(&format!("{}  \"type\": \"Row\",\n", pad));
            out.push_str(&format!("{}  \"align\": \"{}\",\n", pad, align));
            out.push_str(&format!("{}  \"controls\": [\n", pad));
            let nc = ctrl.children.len();
            for (ci, child) in ctrl.children.iter().enumerate() {
                emit_layout_ctrl_json(out, child, indent + 4, ci == nc - 1, counter);
            }
            out.push_str(&format!("{}  ]\n", pad));
            out.push_str(&format!("{}}}{}\n", pad, end));
        }

        ControlKind::Group => {
            let title = ctrl.extra.as_deref().unwrap_or("");
            out.push_str(&format!("{}{{\n", pad));
            out.push_str(&format!("{}  \"type\": \"Group\",\n", pad));
            out.push_str(&format!("{}  \"title\": \"{}\",\n", pad, escape_json_str(title)));
            out.push_str(&format!("{}  \"controls\": [\n", pad));
            let nc = ctrl.children.len();
            for (ci, child) in ctrl.children.iter().enumerate() {
                emit_layout_ctrl_json(out, child, indent + 4, ci == nc - 1, counter);
            }
            out.push_str(&format!("{}  ]\n", pad));
            out.push_str(&format!("{}}}{}\n", pad, end));
        }

        ControlKind::ProgressBar => {
            let id = ctrl.binding.as_deref().map(camel_to_snake).unwrap_or_default();
            let f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"ProgressBar\""),
                format!("\"label\": \"{}\"", escape_json_str(ctrl.label.as_deref().unwrap_or(""))),
            ];
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::StatusBar => {
            let id = ctrl.binding.as_deref().map(camel_to_snake).unwrap_or_default();
            let f: Vec<String> = vec![
                format!("\"id\": \"{}\"", id),
                format!("\"type\": \"StatusBar\""),
            ];
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::Image => {
            let mut f: Vec<String> = vec![
                format!("\"type\": \"Image\""),
                format!("\"source\": \"{}\"", escape_json_str(ctrl.source.as_deref().unwrap_or(""))),
            ];
            if let Some(w) = ctrl.width  { f.push(format!("\"width\": {}", w)); }
            if let Some(h) = ctrl.height { f.push(format!("\"height\": {}", h)); }
            emit_json_fields(out, &pad, &f, end);
        }

        ControlKind::Svg => {
            let mut f: Vec<String> = vec![
                format!("\"type\": \"Svg\""),
                format!("\"source\": \"{}\"", escape_json_str(ctrl.source.as_deref().unwrap_or(""))),
            ];
            if let Some(w) = ctrl.width  { f.push(format!("\"width\": {}", w)); }
            if let Some(h) = ctrl.height { f.push(format!("\"height\": {}", h)); }
            emit_json_fields(out, &pad, &f, end);
        }
    }
}

/// Emit a JSON object from a flat field list.
fn emit_json_fields(out: &mut String, pad: &str, fields: &[String], end: &str) {
    if fields.is_empty() {
        out.push_str(&format!("{}{{}}{}\n", pad, end));
        return;
    }
    out.push_str(&format!("{}{{\n", pad));
    let last = fields.len() - 1;
    for (i, field) in fields.iter().enumerate() {
        let comma = if i < last { "," } else { "" };
        out.push_str(&format!("{}  {}{}\n", pad, field, comma));
    }
    out.push_str(&format!("{}}}{}\n", pad, end));
}

// ---------------------------------------------------------------------------
// Web function body transpiler
// ---------------------------------------------------------------------------

fn transpile_web_fn_body(
    lines: &[String],
    bindings: &[BindingInfo],
    named: &[NamedCtrl],
    value_kind: Option<&BindingKind>,
) -> String {
    let mut out = String::new();
    let mut if_depth = 0usize;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') { continue; }

        // Base indent: 8 spaces (impl method body = 2 levels)
        let indent = format!("        {}", "    ".repeat(if_depth));

        if trimmed.eq_ignore_ascii_case("End If") {
            if if_depth > 0 { if_depth -= 1; }
            let ci = format!("        {}", "    ".repeat(if_depth));
            out.push_str(&format!("{}}}\n", ci));
            continue;
        }
        if trimmed.eq_ignore_ascii_case("Else") {
            if if_depth > 0 { if_depth -= 1; }
            let ci = format!("        {}", "    ".repeat(if_depth));
            out.push_str(&format!("{}}} else {{\n", ci));
            if_depth += 1;
            continue;
        }
        if trimmed.to_lowercase().starts_with("if ") {
            if let Some(cond) = extract_if_condition(trimmed) {
                let cond_rust = web_bool_coerce_expr(&cond, bindings, value_kind);
                out.push_str(&format!("{}if {} {{\n", indent, cond_rust));
                if_depth += 1;
                continue;
            }
        }
        if trimmed.eq_ignore_ascii_case("Exit Function") {
            out.push_str(&format!("{}return serde_wasm_bindgen::to_value(&self.snapshot()).unwrap();\n", indent));
            continue;
        }

        // controlName.Visible = True/False  or  controlName.Caption = "..."
        if let Some(dot_pos) = trimmed.find('.') {
            let obj = &trimmed[..dot_pos];
            let rest = &trimmed[dot_pos + 1..];
            if let Some(eq_pos) = rest.find(" = ") {
                let prop = rest[..eq_pos].trim();
                let val  = rest[eq_pos + 3..].trim();
                if let Some(nc) = named.iter().find(|n| n.vbr_name == obj) {
                    match prop {
                        "Visible" => {
                            let b = val.eq_ignore_ascii_case("true");
                            out.push_str(&format!("{}self.{}_visible = {};\n", indent, nc.id, b));
                            continue;
                        }
                        "Caption" | "Text" => {
                            let stripped = strip_quotes(val);
                            out.push_str(&format!(
                                "{}self.{}_caption = \"{}\".to_string();\n",
                                indent, nc.id, escape_json_str(&stripped)
                            ));
                            continue;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Binding assignment: fieldName = expr  or  data.fieldName = expr
        if let Some((lhs, rhs)) = split_assignment(trimmed) {
            let lhs_bare = lhs.strip_prefix("data.").unwrap_or(&lhs).to_string();
            if let Some(b) = bindings.iter().find(|b| b.camel == lhs_bare || b.snake == lhs_bare) {
                let rhs_rust = transpile_web_rhs(&rhs, b, bindings, value_kind);
                out.push_str(&format!("{}self.{} = {};\n", indent, b.snake, rhs_rust));
                continue;
            }
        }

        out.push_str(&format!("{}// {}\n", indent, trimmed));
    }

    out
}

fn web_bool_coerce_expr(expr: &str, bindings: &[BindingInfo], value_kind: Option<&BindingKind>) -> String {
    let e = expr.trim();
    if !e.is_empty() && e.chars().all(|c| c.is_alphanumeric() || c == '_') {
        if e == "value" {
            return match value_kind {
                Some(BindingKind::Text)    => "!value.is_empty()".to_string(),
                Some(BindingKind::NumberInt | BindingKind::NumberFloat) => "value != 0.0".to_string(),
                _ => e.to_string(),
            };
        }
        if let Some(b) = bindings.iter().find(|b| b.camel == e || b.snake == e) {
            return match b.kind {
                BindingKind::Text        => format!("!self.{}.is_empty()", b.snake),
                BindingKind::NumberInt
                | BindingKind::NumberFloat => format!("self.{} != 0", b.snake),
                _ => format!("self.{}", b.snake),
            };
        }
    }
    transpile_expr_simple(expr)
}

fn transpile_web_rhs(
    rhs: &str,
    binding: &BindingInfo,
    bindings: &[BindingInfo],
    value_kind: Option<&BindingKind>,
) -> String {
    let rhs = rhs.trim();

    // String concat: "prefix" & fieldName & ...
    if rhs.contains(" & ") {
        let parts: Vec<&str> = rhs.split(" & ").collect();
        let fmt = "{}".repeat(parts.len());
        let args: String = parts.iter()
            .map(|p| {
                let p = p.trim();
                if p == "value" {
                    // The handler parameter
                    format!(", {}", match value_kind {
                        Some(BindingKind::Text) => "value".to_string(),
                        _ => "value.to_string()".to_string(),
                    })
                } else if let Some(b) = bindings.iter().find(|b| b.camel == p) {
                    match b.kind {
                        BindingKind::Text => format!(", self.{}", b.snake),
                        _ => format!(", self.{}.to_string()", b.snake),
                    }
                } else {
                    format!(", {}", p)
                }
            })
            .collect();
        return format!("format!(\"{}\"{})", fmt, args);
    }

    // The `value` handler parameter
    if rhs == "value" {
        return match binding.kind {
            BindingKind::Text        => "value.clone()".to_string(),
            BindingKind::NumberInt   => "value as i64".to_string(),
            BindingKind::NumberFloat => "value".to_string(),
            BindingKind::Bool        => "value".to_string(),
            BindingKind::Progress    => "value as f32".to_string(),
        };
    }

    // String literal
    if rhs.starts_with('"') && rhs.ends_with('"') {
        return format!("{}.to_string()", rhs);
    }

    // Another binding reference
    if let Some(b) = bindings.iter().find(|b| b.camel == rhs) {
        return match b.kind {
            BindingKind::Text => format!("self.{}.clone()", b.snake),
            _ => format!("self.{}", b.snake),
        };
    }

    match binding.kind {
        BindingKind::Text => format!("{}.to_string()", rhs),
        _ => rhs.to_string(),
    }
}
