# VBR Forms — Implementation Reference

The forms system spans three crates and the transpiler. This document covers what was built, how the pieces connect, and what the transpiler generates.

---

## Architecture overview

```
VBR source file (.vb)
        │
        ▼
vbr_transpiler (src/form_parser.rs)
        │  generates
        ▼
Rust source file (.rs)
  ├── *Data struct          implements FormData
  ├── *Handlers struct      implements EventDispatch
  ├── build_*_form()        returns FormDef
  └── main()                calls CursiveBackend::run
        │
        ▼
vbr_forms_core              (types and traits only — no UI)
        │
        ▼
vbr_forms_cursive           (TUI backend — cursive crate)
```

The key design principle is that the `Form ... End Form` block in VBR is purely **declarative**. It describes what the form contains; the backend decides how to render it. The same `.vb` file could target a different backend (future `vbr_forms_gpui`, `vbr_forms_web`) without changing a line of VBR source.

---

## vbr_forms_core

Crate path: `vbr_forms_core/`  
No external dependencies.

### Field value

```rust
pub enum FieldValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Progress(f32),
}
```

The common currency between form data and UI controls. Every get/set and every event callback uses `FieldValue`.

### Style enums

```rust
pub enum LabelStyle  { Normal, Bold, Dim }
pub enum ButtonStyle { Normal, Primary, Danger }
pub enum RowAlign    { Left, Center, Right, SpaceBetween }
```

### Control descriptor structs

Each control is described by a plain data struct with a builder pattern. None of these structs contain UI code.

| Struct | Key fields |
|--------|-----------|
| `LabelDef` | `text`, `style: LabelStyle`, `on_click` |
| `TextBoxDef` | `label`, `binding`, `placeholder`, `max_length`, `on_change` |
| `NumberBoxDef` | `label`, `binding`, `min: Option<f64>`, `max: Option<f64>`, `on_change` |
| `CheckBoxDef` | `label`, `binding`, `on_change` |
| `RadioGroupDef` | `label`, `binding`, `options: Vec<String>`, `on_change` |
| `DropDownDef` | `label`, `binding`, `options: Vec<String>`, `on_change` |
| `ButtonDef` | `text`, `on_click`, `style: ButtonStyle` |
| `RowDef` | `align: RowAlign`, `buttons: Vec<ButtonDef>` |
| `GroupDef` | `title`, `controls: Vec<Control>` |
| `ProgressBarDef` | `label`, `binding` |
| `StatusBarDef` | `binding` |

All structs have `::new(required_args)` and builder methods that consume `self` and return `Self`:

```rust
TextBoxDef::new("Host:", "serverHost")
    .placeholder("e.g. localhost")
    .max_length(255)
    .on_change("HostChanged")
```

### Control enum

```rust
pub enum Control {
    Label(LabelDef), Separator,
    TextBox(TextBoxDef), NumberBox(NumberBoxDef),
    CheckBox(CheckBoxDef), RadioGroup(RadioGroupDef), DropDown(DropDownDef),
    Button(ButtonDef), Row(RowDef), Group(GroupDef),
    ProgressBar(ProgressBarDef), StatusBar(StatusBarDef),
}
```

### FormDef

```rust
pub struct FormDef { pub title: String, pub controls: Vec<Control> }
```

Builder: `FormDef::new("Title").add(control).add(control)...`

### Traits

```rust
pub trait FormData: Send + 'static {
    fn get(&self, binding: &str) -> Option<FieldValue>;
    fn set(&mut self, binding: &str, value: FieldValue);
}

pub enum Action { None, Quit }

pub trait EventDispatch: Send + 'static {
    fn dispatch(&mut self, handler: &str, value: FieldValue) -> Action;
}

pub trait FormBackend {
    fn run(
        def: FormDef,
        data: Arc<Mutex<dyn FormData>>,
        events: Arc<Mutex<dyn EventDispatch>>,
    ) -> Result<(), String>;
}
```

`Action::Quit` returned from `dispatch` signals the backend to close the form.

---

## vbr_forms_cursive

Crate path: `vbr_forms_cursive/`  
Dependency: `vbr_forms_core`, `cursive 0.21`

Implements `FormBackend` for the terminal. Entry point:

```rust
pub struct CursiveBackend;

impl FormBackend for CursiveBackend {
    fn run(def: FormDef, data: Arc<Mutex<dyn FormData>>, events: Arc<Mutex<dyn EventDispatch>>)
        -> Result<(), String>
}
```

### Layout model

Top-down vertical flow only. Everything goes into a `LinearLayout::vertical()`. `Row` is the only horizontal escape — it uses `LinearLayout::horizontal()` with `DummyView.full_width()` spacers to implement `SpaceBetween`, `Center`, and `Right` alignment.

There are no coordinates anywhere. This constraint is intentional and carries through to all future backends.

### State sharing

Each input callback captures `Arc<Mutex<dyn FormData>>` and `Arc<Mutex<dyn EventDispatch>>` directly. There is no central event bus.

`AppState` stored in cursive `user_data` carries only the **refresh targets** — the bindings for `StatusBar` and `ProgressBar` controls, which need to be updated after every event:

```rust
struct AppState {
    data: Arc<Mutex<dyn FormData>>,
    refresh_targets: Vec<(String, DisplayKind)>,
}
```

After any input event, the callback calls `refresh_display_views(siv)` which reads those bindings from the data struct and updates the named `TextView` widgets.

### Control rendering

| VBR control | cursive widget |
|-------------|----------------|
| `Label` | `TextView` (styled with `Effect::Bold` / `Effect::Italic`) |
| `Separator` | `DummyView` + `TextView("─"×60)` + `DummyView` |
| `TextBox` | `EditView` in `LinearLayout::horizontal` with a 22-char label cell |
| `NumberBox` | `EditView` (validates `parse::<f64>()`, clamps to min/max) |
| `CheckBox` | `Checkbox` |
| `RadioGroup` | `SelectView<String>` (vertical list) |
| `DropDown` | `SelectView<String>.popup()` |
| `Button` | `Button` |
| `Row` | `LinearLayout::horizontal` with spacers |
| `Group` | `Panel::new(inner_layout).title(...)` |
| `ProgressBar` | `TextView` displaying `label [████░░░░] 42%` |
| `StatusBar` | `TextView` (bold) |

Label cells are fixed at 22 characters wide to align field labels consistently.

Global `q` key quits the form.

### Example

`vbr_forms_cursive/examples/settings.rs` — a hand-written example showing all major controls, the `FormData` impl, the `EventDispatch` impl, and the `main()` wiring.

```bash
cargo run --example settings -p vbr_forms_cursive
```

---

## Form parser and transpiler (src/form_parser.rs)

The transpiler detects `Form "Title"` at the start of a line, routes the file through `form_parser::transpile_form_file`, and writes a complete `.rs` file to stdout.

### VBR form syntax

```vbr
Form "Title"

    Label "text"
        Style := Bold | Dim

    Separator

    Group "Name"
        TextBox
            Label       := "field label"
            Binding     := camelCaseVarName
            Placeholder := "hint text"
            MaxLength   := 255
            OnChange    := HandlerName

        NumberBox
            Label   := "label"
            Binding := varName
            Min     := 1
            Max     := 100

        DropDown
            Label   := "label"
            Options := "A", "B", "C"
            Binding := varName
            OnChange := HandlerName

        RadioGroup
            Label   := "label"
            Options := "X", "Y", "Z"
            Binding := varName
            OnChange := HandlerName

        CheckBox
            Label   := "label"
            Binding := varName
            OnChange := HandlerName
    End Group

    ProgressBar
        Label   := "..."
        Binding := varName

    Row Align := SpaceBetween | Left | Center | Right
        Button "label"
            OnClick := HandlerName
            Style   := Normal | Primary | Danger
    End Row

    StatusBar
        Binding := varName

End Form

Function HandlerName(value As String)
    ' body
End Function
```

**Important parser note:** `Row Align := SpaceBetween` uses inline `:=` syntax. The parser checks that the identifier before `:=` contains no spaces before treating a line as a property assignment — this ensures `Row Align := SpaceBetween` is recognised as a Row start, not a property.

### Generated output structure

Given a form titled `"Connection Setup"`:

**1. Data struct** — one field per `Binding`, typed from the control kind:

```rust
#[derive(Debug, Default)]
struct ConnectionSetupData {
    pub server_host: String,       // TextBox / DropDown / RadioGroup / StatusBar
    pub server_port: i64,          // NumberBox with integer min/max
    pub connection_progress: f32,  // ProgressBar
    pub remember_credentials: bool, // CheckBox
}
```

Binding names are converted from `camelCase` to `snake_case`. `NumberBox` fields use `i64` when `Min`/`Max` have no decimal point, `f64` otherwise.

**2. FormData impl** — `get` and `set` dispatch by binding name string.

**3. Enums for RadioGroup** — each `RadioGroup` with options generates a Rust enum named from the binding (`authType` → `AuthType`), with a `#[default]` on the first variant. (The enum is generated for teaching purposes; the data field is still `String` at runtime.)

**4. Handlers struct + EventDispatch impl:**

```rust
struct ConnectionSetupHandlers {
    data: Arc<Mutex<ConnectionSetupData>>,
}

impl EventDispatch for ConnectionSetupHandlers {
    fn dispatch(&mut self, handler: &str, value: FieldValue) -> Action {
        match handler {
            "HostChanged" => { host_changed(&mut self.data.lock().unwrap(), value); Action::None }
            "Cancel"      => { cancel(&mut self.data.lock().unwrap()); Action::Quit }
            ...
        }
    }
}
```

Handlers whose names contain `cancel`, `close`, `exit`, or `quit` automatically return `Action::Quit`.

**5. FormDef builder:**

```rust
fn build_connection_setup_form() -> FormDef {
    FormDef::new("Connection Setup")
        .add(Control::Label(LabelDef::new("...").style(LabelStyle::Bold)))
        .add(Control::Group(
            GroupDef::new("Server")
            .add(Control::TextBox(TextBoxDef::new("Host:", "serverHost")
                .placeholder("e.g. localhost").max_length(255).on_change("HostChanged")))
        ))
        ...
}
```

**6. Handler function implementations** — one Rust function per handler. Each function that receives a value begins with a `let value = match value { ... }` extraction so the handler body code works with the native type directly:

```rust
fn remember_toggled(data: &mut ConnectionSetupData, value: FieldValue) {
    let value = match value { FieldValue::Bool(b) => b, _ => false };
    if value {
        data.status_message = String::from("Credentials will be saved.");
    } else {
        data.status_message = String::from("Credentials not saved.");
    }
}
```

Function bodies from the VBR source are transpiled:
- `binding = "literal"` → `data.field = String::from("literal")`
- `binding = a & b` → `data.field = format!("{}{}", a, b)`
- `binding = number` → `data.field = number`
- `If expr Then` → `if expr {`
- `Else / End If` → `} else { / }`
- Lines not recognised → emitted as `// comment`

**7. main():**

```rust
fn main() {
    let data = Arc::new(Mutex::new(ConnectionSetupData::default()));
    let handlers = Arc::new(Mutex::new(ConnectionSetupHandlers { data: Arc::clone(&data) }));
    CursiveBackend::run(
        build_connection_setup_form(),
        data as Arc<Mutex<dyn FormData>>,
        handlers as Arc<Mutex<dyn EventDispatch>>,
    ).unwrap();
}
```

### Running the transpiler on a form file

```bash
./target/debug/vbr_transpiler tests/test_form.vb > src/main.rs
```

The output is a standalone `.rs` file. To compile it, create a new Cargo project with `vbr_forms_core` and `vbr_forms_cursive` as dependencies.

---

## Adding a new backend

1. Create a new crate `vbr_forms_<name>` depending on `vbr_forms_core`.
2. Implement `FormBackend` for a unit struct.
3. Walk `FormDef.controls` recursively, building your UI from the descriptor structs.
4. Connect input callbacks using `Arc<Mutex<dyn FormData>>` and `Arc<Mutex<dyn EventDispatch>>` captures.
5. Check `Action::Quit` on every `dispatch` return and close the window if true.
6. Call `refresh_display_views` (or equivalent) after every event to update `StatusBar` / `ProgressBar`.

The generated VBR output only needs its `use vbr_forms_<name>::<Backend>;` import changed to target the new UI framework.
