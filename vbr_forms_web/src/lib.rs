// VBR Forms Web — shared types for WASM/React bridge
//
// This crate provides plain Rust structs that the transpiler-generated WASM
// module uses. The WASM module pulls in vbr_forms_web and serialises these
// types with serde_wasm_bindgen before handing them to React.
//
// No wasm-bindgen here — that belongs in the generated crate so this library
// compiles normally on all targets (useful for testing without a WASM toolchain).

use serde::Serialize;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// The current renderable state of one control.
/// React merges this with the static layout descriptor to produce the UI.
#[derive(Serialize, Clone, Debug, Default)]
pub struct ControlState {
    pub id: String,
    /// Current value as a string (text content, number as decimal, bool as "true"/"false").
    pub value: String,
    pub visible: bool,
    pub enabled: bool,
    /// Dynamic caption for Label and Button controls. None means use the layout text.
    pub caption: Option<String>,
    /// Inline validation error rendered beneath the control.
    pub error: Option<String>,
}

impl ControlState {
    pub fn value(id: impl Into<String>, value: impl Into<String>) -> Self {
        ControlState {
            id: id.into(),
            value: value.into(),
            visible: true,
            enabled: true,
            caption: None,
            error: None,
        }
    }

    pub fn caption(id: impl Into<String>, caption: impl Into<String>) -> Self {
        ControlState {
            id: id.into(),
            value: String::new(),
            visible: true,
            enabled: true,
            caption: Some(caption.into()),
            error: None,
        }
    }
}

/// Complete snapshot of the form at a point in time.
/// React holds one snapshot and treats it as immutable; every WASM call
/// returns a new snapshot.
#[derive(Serialize, Clone, Debug, Default)]
pub struct FormSnapshot {
    pub controls: Vec<ControlState>,
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Reduces snapshot-assembly boilerplate in generated code.
#[derive(Default)]
pub struct SnapshotBuilder {
    controls: Vec<ControlState>,
}

impl SnapshotBuilder {
    pub fn new() -> Self {
        SnapshotBuilder::default()
    }

    pub fn push(mut self, state: ControlState) -> Self {
        self.controls.push(state);
        self
    }

    pub fn build(self) -> FormSnapshot {
        FormSnapshot { controls: self.controls }
    }
}
