// VBR Forms Core
//
// Defines the contract between transpiler-generated code and UI backends.
// No UI framework code lives here — only types and traits.
//
// Architecture:
//   vbr_forms_core  (this crate) — the contract
//   vbr_forms_cursive             — TUI implementation via cursive
//   vbr_forms_gpui (future)       — native GUI via gpui
//   vbr_forms_web  (future)       — web UI
//
// Third parties can publish their own vbr_forms_xyz crates by
// implementing FormBackend and the control builder pattern.

use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Field value — the common currency between data structs and UI controls
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Progress(f32),
}

// ---------------------------------------------------------------------------
// Style enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub enum LabelStyle {
    #[default]
    Normal,
    Bold,
    Dim,
}

#[derive(Debug, Clone, Default)]
pub enum ButtonStyle {
    #[default]
    Normal,
    Primary,
    Danger,
}

#[derive(Debug, Clone, Default)]
pub enum RowAlign {
    #[default]
    Left,
    Center,
    Right,
    SpaceBetween,
}

// ---------------------------------------------------------------------------
// Control descriptor types — declarative, data only, no UI code
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LabelDef {
    pub text: String,
    pub style: LabelStyle,
    pub on_click: Option<String>,
}

impl LabelDef {
    pub fn new(text: impl Into<String>) -> Self {
        LabelDef { text: text.into(), style: LabelStyle::Normal, on_click: None }
    }
    pub fn style(mut self, s: LabelStyle) -> Self { self.style = s; self }
    pub fn on_click(mut self, h: impl Into<String>) -> Self { self.on_click = Some(h.into()); self }
}

#[derive(Debug, Clone)]
pub struct TextBoxDef {
    pub label: String,
    pub binding: String,
    pub placeholder: Option<String>,
    pub max_length: Option<u32>,
    pub on_change: Option<String>,
    pub multi_line: bool,
    pub view_height: Option<u32>,
}

impl TextBoxDef {
    pub fn new(label: impl Into<String>, binding: impl Into<String>) -> Self {
        TextBoxDef {
            label: label.into(),
            binding: binding.into(),
            placeholder: None,
            max_length: None,
            on_change: None,
            multi_line: false,
            view_height: None,
        }
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { self.placeholder = Some(p.into()); self }
    pub fn max_length(mut self, n: u32) -> Self { self.max_length = Some(n); self }
    pub fn on_change(mut self, h: impl Into<String>) -> Self { self.on_change = Some(h.into()); self }
    pub fn multi_line(mut self, v: bool) -> Self { self.multi_line = v; self }
    pub fn view_height(mut self, n: u32) -> Self { self.view_height = Some(n); self }
}

#[derive(Debug, Clone)]
pub struct NumberBoxDef {
    pub label: String,
    pub binding: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub on_change: Option<String>,
}

impl NumberBoxDef {
    pub fn new(label: impl Into<String>, binding: impl Into<String>) -> Self {
        NumberBoxDef {
            label: label.into(),
            binding: binding.into(),
            min: None,
            max: None,
            on_change: None,
        }
    }
    pub fn min(mut self, v: f64) -> Self { self.min = Some(v); self }
    pub fn max(mut self, v: f64) -> Self { self.max = Some(v); self }
    pub fn on_change(mut self, h: impl Into<String>) -> Self { self.on_change = Some(h.into()); self }
}

#[derive(Debug, Clone)]
pub struct CheckBoxDef {
    pub label: String,
    pub binding: String,
    pub on_change: Option<String>,
}

impl CheckBoxDef {
    pub fn new(label: impl Into<String>, binding: impl Into<String>) -> Self {
        CheckBoxDef { label: label.into(), binding: binding.into(), on_change: None }
    }
    pub fn on_change(mut self, h: impl Into<String>) -> Self { self.on_change = Some(h.into()); self }
}

#[derive(Debug, Clone)]
pub struct RadioGroupDef {
    pub label: String,
    pub options: Vec<String>,
    pub binding: String,
    pub on_change: Option<String>,
}

impl RadioGroupDef {
    pub fn new(label: impl Into<String>, binding: impl Into<String>) -> Self {
        RadioGroupDef { label: label.into(), options: Vec::new(), binding: binding.into(), on_change: None }
    }
    pub fn options(mut self, opts: Vec<impl Into<String>>) -> Self {
        self.options = opts.into_iter().map(Into::into).collect();
        self
    }
    pub fn on_change(mut self, h: impl Into<String>) -> Self { self.on_change = Some(h.into()); self }
}

#[derive(Debug, Clone)]
pub struct DropDownDef {
    pub label: String,
    pub options: Vec<String>,
    pub binding: String,
    pub on_change: Option<String>,
}

impl DropDownDef {
    pub fn new(label: impl Into<String>, binding: impl Into<String>) -> Self {
        DropDownDef { label: label.into(), options: Vec::new(), binding: binding.into(), on_change: None }
    }
    pub fn options(mut self, opts: Vec<impl Into<String>>) -> Self {
        self.options = opts.into_iter().map(Into::into).collect();
        self
    }
    pub fn on_change(mut self, h: impl Into<String>) -> Self { self.on_change = Some(h.into()); self }
}

#[derive(Debug, Clone)]
pub struct ButtonDef {
    pub text: String,
    pub on_click: String,
    pub style: ButtonStyle,
}

impl ButtonDef {
    pub fn new(text: impl Into<String>, on_click: impl Into<String>) -> Self {
        ButtonDef { text: text.into(), on_click: on_click.into(), style: ButtonStyle::Normal }
    }
    pub fn style(mut self, s: ButtonStyle) -> Self { self.style = s; self }
}

#[derive(Debug, Clone)]
pub struct RowDef {
    pub align: RowAlign,
    pub buttons: Vec<ButtonDef>,
}

impl RowDef {
    pub fn new(align: RowAlign) -> Self {
        RowDef { align, buttons: Vec::new() }
    }
    pub fn add(mut self, btn: ButtonDef) -> Self { self.buttons.push(btn); self }
}

#[derive(Debug, Clone)]
pub struct GroupDef {
    pub title: String,
    pub controls: Vec<Control>,
}

impl GroupDef {
    pub fn new(title: impl Into<String>) -> Self {
        GroupDef { title: title.into(), controls: Vec::new() }
    }
    pub fn add(mut self, c: Control) -> Self { self.controls.push(c); self }
}

#[derive(Debug, Clone)]
pub struct ProgressBarDef {
    pub label: String,
    pub binding: String,
}

impl ProgressBarDef {
    pub fn new(label: impl Into<String>, binding: impl Into<String>) -> Self {
        ProgressBarDef { label: label.into(), binding: binding.into() }
    }
}

#[derive(Debug, Clone)]
pub struct StatusBarDef {
    pub binding: String,
}

impl StatusBarDef {
    pub fn new(binding: impl Into<String>) -> Self {
        StatusBarDef { binding: binding.into() }
    }
}

// ---------------------------------------------------------------------------
// Control enum — the complete V1 control set
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Control {
    Label(LabelDef),
    Separator,
    TextBox(TextBoxDef),
    NumberBox(NumberBoxDef),
    CheckBox(CheckBoxDef),
    RadioGroup(RadioGroupDef),
    DropDown(DropDownDef),
    Button(ButtonDef),
    Row(RowDef),
    Group(GroupDef),
    ProgressBar(ProgressBarDef),
    StatusBar(StatusBarDef),
}

// ---------------------------------------------------------------------------
// Form definition — the root descriptor
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FormDef {
    pub title: String,
    pub controls: Vec<Control>,
}

impl FormDef {
    pub fn new(title: impl Into<String>) -> Self {
        FormDef { title: title.into(), controls: Vec::new() }
    }
    pub fn add(mut self, c: Control) -> Self { self.controls.push(c); self }
}

// ---------------------------------------------------------------------------
// Binding traits — the data contract
// ---------------------------------------------------------------------------

/// Implemented by transpiler-generated form data structs.
/// Maps binding names to typed field values.
pub trait FormData: Send + 'static {
    fn get(&self, binding: &str) -> Option<FieldValue>;
    fn set(&mut self, binding: &str, value: FieldValue);
}

/// Return value from event dispatch — lets handlers signal backend actions.
#[derive(Debug, Default)]
pub enum Action {
    #[default]
    None,
    /// Close the form and stop the event loop
    Quit,
}

/// Implemented by the user's event handler struct.
/// `dispatch` is called with the handler name and the triggering value.
pub trait EventDispatch: Send + 'static {
    fn dispatch(&mut self, handler: &str, value: FieldValue) -> Action;
}

// ---------------------------------------------------------------------------
// Backend trait — implemented by each UI framework crate
// ---------------------------------------------------------------------------

/// The single trait a UI backend must implement.
/// Takes a FormDef + data + event dispatcher and runs the UI event loop.
pub trait FormBackend {
    fn run(
        def: FormDef,
        data: Arc<Mutex<dyn FormData>>,
        events: Arc<Mutex<dyn EventDispatch>>,
    ) -> Result<(), String>;
}
