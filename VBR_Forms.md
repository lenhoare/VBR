## The Core Forms Principle

```
VBR source
    │
    ▼
Transpiler  →  Generic framework agnostic Rust
                        │
                        ▼
              vbr_forms_core  —  defines the interface
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
        vbr_tui     vbr_gpui    vbr_anything
    (cursive)      (gpui)      (third party)
```

The transpiler never changes. The core interface never changes. Anyone can write a new UI crate that slots in.

---

## What The Transpiler Generates

Generic Rust using traits from `vbr_forms_core`:

```rust
use vbr_forms_core::{Form, Label, TextBox, Button, EventHandler};

struct FrmMain {
    lbl_name: Label,
    txt_name: TextBox,
    btn_ok: Button,
}

impl Form for FrmMain {
    fn new() -> Self {
        FrmMain {
            lbl_name: Label::new("Name:", 2, 2),
            txt_name: TextBox::new(2, 10, 30),
            btn_ok: Button::new("OK", 20, 35, 10),
        }
    }

    fn title(&self) -> &str { "My Application" }
    fn width(&self) -> u32 { 80 }
    fn height(&self) -> u32 { 24 }
}

impl EventHandler for FrmMain {
    fn btn_ok_click(&mut self) {
        let name = self.txt_name.value();
        println!("Hello {}", name);
    }
}
```

This Rust compiles and runs with ANY UI framework that implements `vbr_forms_core` traits.

---

## The vbr_forms_core Crate

This is the **contract** — the interface every UI framework must implement:

```rust
// vbr_forms_core/src/lib.rs

pub trait Form {
    fn new() -> Self;
    fn title(&self) -> &str;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn show(&mut self);
}

pub trait EventHandler {
    // Default empty implementations
    // so users only implement what they need
    fn form_load(&mut self) {}
    fn form_close(&mut self) {}
}

pub trait Label {
    fn new(caption: &str, top: u32, left: u32) -> Self;
    fn caption(&self) -> &str;
    fn set_caption(&mut self, caption: &str);
    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
}

pub trait TextBox {
    fn new(top: u32, left: u32, width: u32) -> Self;
    fn value(&self) -> String;
    fn set_value(&mut self, value: &str);
    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

pub trait Button {
    fn new(caption: &str, top: u32, left: u32, width: u32) -> Self;
    fn caption(&self) -> &str;
    fn set_enabled(&mut self, enabled: bool);
}

pub trait CheckBox {
    fn new(caption: &str, top: u32, left: u32) -> Self;
    fn value(&self) -> bool;
    fn set_value(&mut self, value: bool);
}

pub trait Select {
    fn new(top: u32, left: u32, width: u32) -> Self;
    fn add_item(&mut self, item: &str);
    fn selected(&self) -> Option<String>;
    fn set_selected(&mut self, item: &str);
}

pub trait ListBox {
    fn new(top: u32, left: u32, width: u32, height: u32) -> Self;
    fn add_item(&mut self, item: &str);
    fn selected(&self) -> Option<String>;
    fn clear(&mut self);
}

// Built in dialogs — every framework must implement these
pub trait Dialogs {
    fn msgbox(message: &str, title: &str);
    fn confirm(message: &str) -> bool;
    fn inputbox(prompt: &str) -> Option<String>;
    fn open_file(title: &str) -> Option<String>;
    fn save_file(title: &str) -> Option<String>;
}
```

---

## The vbr_tui Crate

Implements `vbr_forms_core` traits using cursive:

```rust
// vbr_tui/src/lib.rs
use vbr_forms_core::{TextBox as TextBoxTrait};
use cursive::views::EditView;

pub struct TextBox {
    inner: EditView,
    top: u32,
    left: u32,
    width: u32,
}

impl TextBoxTrait for TextBox {
    fn new(top: u32, left: u32, width: u32) -> Self {
        TextBox {
            inner: EditView::new(),
            top,
            left,
            width,
        }
    }

    fn value(&self) -> String {
        self.inner.get_content().to_string()
    }

    fn set_value(&mut self, value: &str) {
        self.inner.set_content(value);
    }

    fn enabled(&self) -> bool {
        self.inner.is_enabled()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.inner.set_enabled(enabled);
    }
}
```

---

## A Third Party UI Framework

Anyone can create `vbr_myframework`:

```rust
// vbr_myframework/src/lib.rs
use vbr_forms_core::TextBox as TextBoxTrait;

pub struct TextBox {
    // their own implementation
}

impl TextBoxTrait for TextBox {
    fn new(top: u32, left: u32, width: u32) -> Self {
        // their implementation
    }
    // implement all required methods
}
```

They publish it to crates.io. VBR users just add:

```vba
Use VBR::Forms::MyFramework
```

The transpiler never changes. The generated Rust never changes. Only the UI crate changes.

---

## The User's Cargo.toml

Generated automatically by the transpiler based on the `Use` statement:

```toml
' Use VBR::Forms::TUI
[dependencies]
vbr_forms_core = "0.1"
vbr_tui = "0.1"

' Use VBR::Forms::GPUI
[dependencies]
vbr_forms_core = "0.1"
vbr_gpui = "0.1"

' Use VBR::Forms::MyFramework
[dependencies]
vbr_forms_core = "0.1"
vbr_myframework = "0.1"
```

---

## The Complete Project Structure

```
vbr_transpiler/       — transpiler, never changes for UI
vbr_stdlib/           — FileSystem, Json, Http etc
vbr_forms_core/       — trait definitions, the contract
vbr_tui/              — cursive implementation
vbr_gpui/             — gpui implementation (later)
```

Third parties publish their own `vbr_xyz` crates independently.

---

## What The Transpiler Needs To Know

Just three things — and none of them are framework specific:

1. `Use VBR::Forms::X` → add `vbr_forms_core` and `vbr_x` to Cargo.toml
2. `Form...End Form` → generate struct implementing `Form` trait
3. `controlName_EventName` functions → generate `EventHandler` impl

That's it. The transpiler is completely insulated from UI framework details forever.

---

This is a genuinely good architecture. Clean separation, extensible, third party friendly. Does this feel right?