# VBR TUI Specification

## Overview

VBR's TUI (Terminal User Interface) follows a **top-down flow model** — controls are declared in order and render vertically, like a markdown document. There is no visual designer. The developer specifies controls and they stack naturally from top to bottom.

The only exception to vertical flow is the `Row` container, which creates a horizontal group of controls as a single block within the vertical flow.

This model maps cleanly across devices and terminal sizes, and translates naturally to other UI targets (web, native) when additional framework crates are added.

The TUI target is implemented via the `cursive` crate. The transpiler generates framework-agnostic Rust; `vbr_forms_cursive` is the V1 UI crate that implements the `vbr_forms_core` traits.

---

## Core Concepts

### Binding

Every input control connects to exactly one Rust variable via `Binding`. The transpiler generates a struct from the form's bindings. Data lives in the struct — there is no `TextBox.Value` style access. This is the fundamental teaching point of VBR's UI model.

### OnChange vs OnClick

- `Binding` handles **state sync** — the struct field is updated automatically when the user changes a control.
- `OnChange` handles **reactions** — a function called after the binding is updated, receiving the new value as a parameter.
- `OnClick` handles **actions** — triggered by buttons and optionally by labels.

These are deliberately separate concerns. Binding is data. Events are behaviour.

### Generated Event Handlers

Event handler functions receive the new value directly as a parameter. There is no need to read the control — the value is passed in. This is idiomatic Rust.

```vbr
Function ProxyToggled(value As Bool)
    ' react to the new state
End Function
```

---

## Control Reference

### Label
Static text. Supports an optional click handler for link-style interactions.

| Property | Type | Notes |
|----------|------|-------|
| `Text` | String | The text to display |
| `Style` | `Normal \| Bold \| Dim` | Visual hierarchy |
| `OnClick` | Function name | Optional. Makes the label interactive |

---

### Separator
A horizontal rule. No properties. Used for visual grouping.

---

### TextBox
Single-line text input.

| Property | Type | Notes |
|----------|------|-------|
| `Label` | String | Inline label shown before the field |
| `Placeholder` | String | Hint text shown when empty |
| `MaxLength` | Integer | Optional character cap |
| `Binding` | Variable name | Maps to a `String` field in the generated struct |
| `OnChange` | Function name | Optional. Called with new `String` value |

---

### NumberBox
Numeric input. Maps to a typed Rust numeric field.

| Property | Type | Notes |
|----------|------|-------|
| `Label` | String | Inline label |
| `Min` | Number | Optional minimum value |
| `Max` | Number | Optional maximum value |
| `Binding` | Variable name | Maps to `i64` or `f64` depending on `Min`/`Max` values |
| `OnChange` | Function name | Optional. Called with new numeric value |

---

### CheckBox
Boolean toggle.

| Property | Type | Notes |
|----------|------|-------|
| `Label` | String | Text shown beside the checkbox |
| `Binding` | Variable name | Maps to `bool` |
| `OnChange` | Function name | Optional. Called with new `bool` value |

---

### RadioGroup
Mutually exclusive options. The transpiler generates a Rust `enum` from the options list.

| Property | Type | Notes |
|----------|------|-------|
| `Label` | String | Group label shown above the options |
| `Options` | String list | The option values; transpiler generates a `enum` |
| `Binding` | Variable name | Maps to the generated `enum` type |
| `OnChange` | Function name | Optional. Called with new enum value |

---

### DropDown
Select one value from a list.

| Property | Type | Notes |
|----------|------|-------|
| `Label` | String | Inline label |
| `Options` | String list or bound `Vec` | Static list or dynamic binding |
| `Binding` | Variable name | Maps to `String` or enum |
| `OnChange` | Function name | Optional. Called with new value |

---

### Button
Triggers a function. Does not bind to data.

| Property | Type | Notes |
|----------|------|-------|
| `Text` | String | Button label |
| `OnClick` | Function name | Required. The function to call |
| `Style` | `Normal \| Primary \| Danger` | Visual intent |

---

### Row
Horizontal layout container. A single block in the vertical flow that arranges its children left to right.

| Property | Type | Notes |
|----------|------|-------|
| `Align` | `Left \| Center \| Right \| SpaceBetween` | Controls horizontal placement of children |

Contains: `Button` controls only in V1.

---

### Group
A labeled border around a logical section of controls.

| Property | Type | Notes |
|----------|------|-------|
| `Title` | String | The border label |

Contains: any controls.

---

### ProgressBar
Visual feedback for long-running operations.

| Property | Type | Notes |
|----------|------|-------|
| `Label` | String | Text shown beside or above the bar |
| `Binding` | Variable name | Maps to `f32` between `0.0` and `1.0` |

---

### StatusBar
A single line at the bottom of the form showing current state. There is one `StatusBar` per form.

| Property | Type | Notes |
|----------|------|-------|
| `Binding` | Variable name | Maps to `String` |

---

## What V1 Deliberately Excludes

These controls are deferred to V2 or later:

| Control | Reason |
|---------|--------|
| `ListBox` | `DropDown` handles selection; scrollable lists are V2 |
| `TreeView` / `ListView` | V2 territory |
| `TabControl` | Encourages VB6 thinking; use `Group` blocks or multiple forms |
| `MultiLineTextBox` | Opens significant complexity; V2 |
| `Slider` | Niche use case; V2 |
| `DatePicker` | `DateTime` stdlib handles dates in code |

---

## Complete Example

The following VBR form demonstrates every V1 control.

```vbr
Form "ConnectionSetup"

    ' --- Header ---
    Label "VBR Connection Setup"
        Style := Bold

    Label "Configure your server connection below."
        Style := Dim

    Separator

    ' --- Server details ---
    Group "Server"

        TextBox
            Label   := "Host:"
            Placeholder := "e.g. localhost"
            MaxLength   := 255
            Binding := serverHost
            OnChange := HostChanged

        NumberBox
            Label   := "Port:"
            Min     := 1
            Max     := 65535
            Binding := serverPort

        DropDown
            Label   := "Protocol:"
            Options := "HTTP", "HTTPS", "FTP"
            Binding := serverProtocol
            OnChange := ProtocolChanged

    End Group

    ' --- Authentication ---
    Group "Authentication"

        RadioGroup
            Label   := "Auth Type:"
            Options := "None", "Basic", "Token"
            Binding := authType
            OnChange := AuthTypeChanged

        TextBox
            Label   := "Username:"
            Binding := username

        TextBox
            Label   := "Password:"
            Binding := password

        CheckBox
            Label   := "Remember credentials"
            Binding := rememberCredentials
            OnChange := RememberToggled

    End Group

    ' --- Proxy ---
    Group "Proxy"

        CheckBox
            Label   := "Use proxy"
            Binding := useProxy
            OnChange := ProxyToggled

        TextBox
            Label   := "Proxy host:"
            Binding := proxyHost

        NumberBox
            Label   := "Proxy port:"
            Min     := 1
            Max     := 65535
            Binding := proxyPort

    End Group

    ' --- Progress and status ---
    ProgressBar
        Label   := "Testing connection..."
        Binding := connectionProgress

    ' --- Clickable link-style label ---
    Label "Need help? View documentation"
        Style   := Dim
        OnClick := OpenDocs

    Separator

    ' --- Action buttons ---
    Row Align := SpaceBetween

        Button "Test Connection"
            OnClick := TestConnection
            Style   := Normal

        Button "Connect"
            OnClick := Connect
            Style   := Primary

        Button "Cancel"
            OnClick := Cancel
            Style   := Danger

    End Row

    ' --- Status bar at bottom ---
    StatusBar
        Binding := statusMessage

End Form


' --- Event handlers ---

Function HostChanged(value As String)
End Function

Function ProtocolChanged(value As String)
End Function

Function AuthTypeChanged(value As AuthType)
End Function

Function RememberToggled(value As Bool)
End Function

Function ProxyToggled(value As Bool)
End Function

Function TestConnection()
End Function

Function Connect()
End Function

Function Cancel()
End Function

Function OpenDocs()
End Function
```

### Generated Struct (illustrative)

The transpiler produces a Rust struct from all `Binding` declarations on the form:

```rust
pub struct ConnectionSetupForm {
    pub server_host: String,
    pub server_port: i64,
    pub server_protocol: String,
    pub auth_type: AuthType,
    pub username: String,
    pub password: String,
    pub remember_credentials: bool,
    pub use_proxy: bool,
    pub proxy_host: String,
    pub proxy_port: i64,
    pub connection_progress: f32,
    pub status_message: String,
}

pub enum AuthType {
    None,
    Basic,
    Token,
}
```

Note that `RadioGroup` with `Options := "None", "Basic", "Token"` causes the transpiler to generate the `AuthType` enum automatically. The enum name is derived from the `Binding` variable name.

---

## Design Principles

- **One rule, one exception**: everything flows top to bottom; `Row` is the only horizontal escape hatch.
- **Binding is data, events are behaviour**: these are always separate, never tangled.
- **Minimal surface area**: 12 controls, a small set of properties per control, two event types.
- **Teach Rust idioms**: `bool` for checkboxes, generated enums for radio groups, `f32` for progress, no stringly-typed control access.
- **Framework-agnostic core**: the transpiler generates Rust against `vbr_forms_core` traits; `vbr_forms_cursive` is the V1 TUI implementation. Other UI crates can be published independently.
