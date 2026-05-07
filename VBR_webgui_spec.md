# VBR Web GUI — WASM + React Architecture Spec

**Version:** 0.1  
**Status:** Draft  
**Scope:** V1 — synchronous pull model only

---

## 1. Overview

VBR supports a web GUI compilation target that compiles Rust form logic to WebAssembly (WASM) and drives a React-based rendering shell. The VBR programmer writes only Rust — no JavaScript, no JSX, no React concepts. The transpiler and the `vbr_forms_web` crate handle the bridge.

The guiding constraint: **Rust owns all state. React is a pure rendering layer.**

---

## 2. Architecture

```
┌──────────────────────────────────────────────┐
│                  Browser                      │
│                                               │
│  ┌─────────────────┐    ┌──────────────────┐  │
│  │   React Shell   │    │   WASM Module    │  │
│  │   (generated)   │◄──►│  (compiled Rust) │  │
│  │                 │    │                  │  │
│  │  Renders UI     │    │  Owns all state  │  │
│  │  Handles DOM    │    │  All logic       │  │
│  │  events         │    │  Event handlers  │  │
│  └────────┬────────┘    └──────────────────┘  │
│           │                                   │
│  ┌────────▼────────┐                          │
│  │  Layout         │                          │
│  │  Descriptor     │                          │
│  │  (generated TS) │                          │
│  └─────────────────┘                          │
└──────────────────────────────────────────────┘
```

### Components

| Component | Language | Responsibility |
|---|---|---|
| WASM Module | Rust (compiled) | All state, all logic, all event handlers |
| Layout Descriptor | TypeScript (generated) | Declarative UI structure — no logic |
| React Shell | TypeScript (static, generic) | Reads descriptor, wires events, renders |
| `vbr_forms_web` | Rust crate | `wasm-bindgen` glue, snapshot serialisation |

The React Shell is **not generated per-form**. It is a single generic component
shipped with `vbr_forms_web` that interprets any layout descriptor. Only the
descriptor and the WASM module are form-specific outputs.

---

## 3. State Model — Pull Only (V1)

Every state change is **synchronous and event-driven**:

1. User triggers a DOM event (click, input change, etc.)
2. React calls the corresponding WASM method
3. WASM updates internal state and returns a `FormSnapshot`
4. React replaces its snapshot reference and re-renders

There is no push path in V1. Rust never initiates a state update. All UI
changes are a direct response to user events. Async/push is deferred to V2.

### FormSnapshot

The `FormSnapshot` is a plain serialisable struct that represents the complete
renderable state of a form at a point in time. React holds exactly one snapshot
at a time and treats it as immutable.

```rust
// vbr_forms_web runtime (not transpiler output)
#[wasm_bindgen]
#[derive(Serialize)]
pub struct FormSnapshot {
    pub controls: Vec<ControlState>,
}

#[derive(Serialize)]
pub struct ControlState {
    pub id: String,
    pub value: String,       // All values are strings at the boundary
    pub visible: bool,
    pub enabled: bool,
    pub label: Option<String>,
    pub error: Option<String>,
}
```

`FormSnapshot` is serialised to a plain JS object via `serde-wasm-bindgen`
before being handed to React. React never receives a live Rust reference.

---

## 4. Transpiler Output

For a given VBR form file, the transpiler emits two artefacts:

### 4.1 Rust/WASM Module (`my_form.rs` → compiled to `my_form.wasm`)

The transpiler generates a `wasm-bindgen`-annotated Rust struct with one public
method per form event. Each method takes event-specific parameters (if any),
updates internal state, and returns a `FormSnapshot`.

```rust
// Generated output — illustrative
use wasm_bindgen::prelude::*;
use vbr_forms_web::FormSnapshot;

#[wasm_bindgen]
pub struct MyForm {
    name: String,
    count: i32,
}

#[wasm_bindgen]
impl MyForm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MyForm {
        MyForm { name: String::new(), count: 0 }
    }

    // Called once on load to get the initial render state
    pub fn initial_snapshot(&self) -> FormSnapshot {
        self.snapshot()
    }

    // Event handler — returns updated snapshot
    pub fn btn_submit_click(&mut self) -> FormSnapshot {
        self.count += 1;
        self.snapshot()
    }

    // Event handler with value payload from input control
    pub fn txt_name_change(&mut self, value: String) -> FormSnapshot {
        self.name = value;
        self.snapshot()
    }

    fn snapshot(&self) -> FormSnapshot {
        // builds and returns FormSnapshot from current state
        todo!()
    }
}
```

**Rules:**
- Every event handler returns `FormSnapshot` — no exceptions
- No event handler is `async` in V1
- `initial_snapshot()` is always generated and called by React on mount
- The struct name matches the form name in `PascalCase`
- All event handler names are `snake_case`

### 4.2 Layout Descriptor (`my_form_layout.ts`)

A static TypeScript object describing the form's controls and which WASM method
each event maps to. Contains no logic.

```typescript
// Generated output — illustrative
export const MyFormLayout: FormLayout = {
  title: "My Form",
  controls: [
    {
      id: "txt_name",
      type: "TextBox",
      label: "Name",
      events: {
        onChange: "txt_name_change"   // maps to WASM method name
      }
    },
    {
      id: "lbl_count",
      type: "Label",
      label: "Count:"
    },
    {
      id: "btn_submit",
      type: "Button",
      label: "Submit",
      events: {
        onClick: "btn_submit_click"   // maps to WASM method name
      }
    }
  ]
};
```

**Rules:**
- All `id` values are `snake_case` matching the WASM `ControlState` ids
- Event method names are plain strings matching WASM export names exactly
- No TypeScript logic — no functions, no conditionals, no imports beyond types
- Control types are constrained to the V1 control set (see Section 6)

---

## 5. React Shell Contract

The generic React Shell (`<VbrForm />`) accepts:

```typescript
interface VbrFormProps {
  wasmModule: MyForm;        // instantiated WASM struct
  layout: FormLayout;        // generated layout descriptor
}
```

On mount:
1. Calls `wasmModule.initial_snapshot()` to get initial state
2. Merges snapshot with layout to produce render tree
3. Renders controls

On event:
1. Looks up the event's method name from the layout descriptor
2. Calls `wasmModule[methodName](payload?)` — payload is a string for `onChange`, absent for `onClick`
3. Receives new `FormSnapshot`
4. Replaces snapshot state, triggers re-render

The shell merges `ControlState` (from snapshot) with the layout entry by `id`
to produce each rendered control. Layout provides structure; snapshot provides
current values and visibility.

```
Layout[id]  +  Snapshot.controls[id]  →  Rendered control
```

If a control `id` exists in the layout but not in the snapshot, it renders with
defaults (`visible: true`, `enabled: true`, empty value). This is a transpiler
bug, not a runtime feature.

---

## 6. V1 Control Set

| VBR Control | Rendered As | Events | Snapshot Fields Used |
|---|---|---|---|
| `TextBox` | `<input type="text">` | `onChange` | `value`, `enabled`, `visible`, `error` |
| `Label` | `<span>` | — | `label`, `visible` |
| `Button` | `<button>` | `onClick` | `label`, `enabled`, `visible` |
| `CheckBox` | `<input type="checkbox">` | `onChange` | `value` (`"true"`/`"false"`), `enabled`, `visible`, `label` |
| `ComboBox` | `<select>` | `onChange` | `value`, `enabled`, `visible`; options from layout only (V1) |

All values crossing the WASM boundary are `String`. The shell and the Rust
struct share the same encoding convention per control type. Booleans are
`"true"`/`"false"`. Numeric values are their decimal string representation.

---

## 7. Crate: `vbr_forms_web`

A real Rust crate (not transpiler rules) that provides:

- `FormSnapshot` and `ControlState` types with `wasm-bindgen` and `serde` derives
- A `SnapshotBuilder` helper to reduce boilerplate in generated form code
- TypeScript type definitions (via `wasm-bindgen`'s TS generation) for `FormLayout`, `FormControl`, `ControlEvent`
- The static `<VbrForm />` React component and its rendering logic (shipped as a companion npm package `vbr-forms-web`)

### Rust dependencies

```toml
[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

### npm package: `vbr-forms-web`

Ships alongside the crate. Contains:
- `VbrForm` React component
- TypeScript types for `FormLayout`, `ControlState`, `FormSnapshot`
- No styling opinions in V1 — consumers apply their own CSS or a theme package

---

## 8. Build Pipeline

```
VBR source file
      │
      ▼
vbr_transpiler
      │
      ├──► my_form.rs          (Rust with wasm-bindgen annotations)
      │
      └──► my_form_layout.ts   (TypeScript layout descriptor)

my_form.rs + vbr_forms_web
      │
      ▼
wasm-pack build
      │
      ├──► my_form_bg.wasm
      └──► my_form.js + my_form.d.ts   (wasm-bindgen JS glue)

Consumer app
      │
      ├── imports my_form.js (WASM glue)
      ├── imports my_form_layout.ts
      ├── imports VbrForm from vbr-forms-web
      └── renders <VbrForm wasmModule={...} layout={MyFormLayout} />
```

Build tooling: `wasm-pack` is the recommended build tool. Output targets
`bundler` mode for Webpack/Vite compatibility.

---

## 9. Error Handling

V1 error handling at the WASM boundary is minimal and explicit:

- Rust panics are caught by `wasm-bindgen` and surfaced as JS exceptions
- The React Shell wraps WASM calls in `try/catch`; on exception it renders a
  visible error state rather than silently failing
- Per-control validation errors are surfaced via `ControlState.error: Option<String>`
  — the shell renders these inline beneath the relevant control
- No error recovery or retry logic in V1

---

## 10. Explicit Non-Goals (V1)

- No async event handlers — all WASM calls are synchronous
- No push/notification path from Rust to React
- No ComboBox options from WASM — options are layout-only in V1
- No styling or theming — `vbr-forms-web` ships unstyled
- No form-to-form navigation
- No Office interop
- No `wasm-bindgen` multi-threading

---

## 11. Open Questions

| # | Question | Notes |
|---|---|---|
| 1 | Should `ControlState.value` be `Option<String>` or always `String`? | Always `String` is simpler at the boundary; `Option` adds clarity for unset controls |
| 2 | How are ComboBox options provided from WASM in V2? | Probably a `Vec<String>` field on `ControlState` |
| 3 | Should the layout descriptor be JSON instead of TypeScript? | TS has type safety; JSON is more portable. Decide before V1 freeze |
| 4 | `wasm-pack` target: `bundler` vs `web`? | `bundler` requires Webpack/Vite; `web` works with native ESM but needs async init |
