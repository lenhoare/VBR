// VBR Forms Web — TypeScript types for the React shell
//
// These match the Rust structs in vbr_forms_web/src/lib.rs and the JSON
// layout format emitted by the VBR transpiler.

// ---------------------------------------------------------------------------
// Snapshot types (from WASM)
// ---------------------------------------------------------------------------

export interface ControlState {
  id: string;
  value: string;
  visible: boolean;
  enabled: boolean;
  caption: string | null;
  error: string | null;
}

export interface FormSnapshot {
  controls: ControlState[];
}

// ---------------------------------------------------------------------------
// Layout types (from generated JSON)
// ---------------------------------------------------------------------------

export interface ControlEvents {
  onClick?: string;
  onChange?: string;
}

export interface ControlDef {
  /** snake_case identifier, matches ControlState.id */
  id?: string;
  type: string;
  /** Static text for Label / Button (overridden by ControlState.caption if set) */
  text?: string;
  /** Field label for TextBox, NumberBox, etc. */
  label?: string;
  style?: string;
  placeholder?: string;
  multiLine?: boolean;
  viewHeight?: number;
  min?: number;
  max?: number;
  /** Options for RadioGroup and DropDown */
  options?: string[];
  /** Image / Svg source path or URL */
  source?: string;
  width?: number;
  height?: number;
  /** Row alignment: "Left" | "Center" | "Right" | "SpaceBetween" */
  align?: string;
  /** Group title */
  title?: string;
  /** Nested controls (Group, Row) */
  controls?: ControlDef[];
  events?: ControlEvents;
}

export interface FormLayout {
  title: string;
  controls: ControlDef[];
}
