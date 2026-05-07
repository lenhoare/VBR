// VBR Forms Web — generic React shell
//
// Usage:
//   import init, { MyForm } from './my_form';
//   import MyFormLayout from './my_form.layout.json';
//   import { VbrForm } from 'vbr-forms-web/react/VbrForm';
//
//   await init();
//   const module = new MyForm();
//   <VbrForm wasmModule={module} layout={MyFormLayout} />
//
// Rust owns all state. This component is a pure rendering layer that calls
// WASM methods and re-renders on each response.

import React, { useEffect, useState, useCallback } from 'react';
import { ControlDef, ControlState, FormLayout, FormSnapshot } from './types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface VbrFormProps {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  wasmModule: any;
  layout: FormLayout;
}

// ---------------------------------------------------------------------------
// Default state for controls not in the snapshot
// ---------------------------------------------------------------------------

const defaultState = (id: string): ControlState => ({
  id,
  value: '',
  visible: true,
  enabled: true,
  caption: null,
  error: null,
});

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function VbrForm({ wasmModule, layout }: VbrFormProps) {
  const [snapshot, setSnapshot] = useState<FormSnapshot>({ controls: [] });

  useEffect(() => {
    const initial: FormSnapshot = wasmModule.initial_snapshot();
    setSnapshot(initial);
  }, [wasmModule]);

  const getState = useCallback(
    (id: string): ControlState =>
      snapshot.controls.find(c => c.id === id) ?? defaultState(id),
    [snapshot],
  );

  const dispatch = useCallback(
    (method: string, payload?: string | boolean | number) => {
      try {
        const result: FormSnapshot =
          payload !== undefined ? wasmModule[method](payload) : wasmModule[method]();
        setSnapshot(result);
      } catch (e) {
        console.error(`VBR dispatch error calling ${method}:`, e);
      }
    },
    [wasmModule],
  );

  return (
    <div className="vbr-form">
      <h2 className="vbr-form-title">{layout.title}</h2>
      {renderControls(layout.controls, getState, dispatch)}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Control renderers
// ---------------------------------------------------------------------------

function renderControls(
  controls: ControlDef[],
  getState: (id: string) => ControlState,
  dispatch: (method: string, payload?: string | boolean | number) => void,
): React.ReactNode[] {
  return controls.map((ctrl, i) => renderControl(ctrl, i, getState, dispatch));
}

function renderControl(
  ctrl: ControlDef,
  key: number,
  getState: (id: string) => ControlState,
  dispatch: (method: string, payload?: string | boolean | number) => void,
): React.ReactNode {
  const state = ctrl.id ? getState(ctrl.id) : null;

  if (state && !state.visible) return null;

  switch (ctrl.type) {

    case 'Label': {
      const cls = ['vbr-label', ctrl.style?.toLowerCase()].filter(Boolean).join(' ');
      return (
        <div key={key} className={cls}>
          {state?.caption ?? ctrl.text}
        </div>
      );
    }

    case 'Separator':
      return <hr key={key} className="vbr-separator" />;

    case 'TextBox': {
      const val = state?.value ?? '';
      return (
        <div key={key} className="vbr-field">
          <label className="vbr-field-label">{ctrl.label}</label>
          {ctrl.multiLine ? (
            <textarea
              className="vbr-textarea"
              value={val}
              rows={ctrl.viewHeight ?? 4}
              disabled={!(state?.enabled ?? true)}
              onChange={e => ctrl.events?.onChange && dispatch(ctrl.events.onChange, e.target.value)}
            />
          ) : (
            <input
              type="text"
              className="vbr-input"
              value={val}
              placeholder={ctrl.placeholder}
              disabled={!(state?.enabled ?? true)}
              onChange={e => ctrl.events?.onChange && dispatch(ctrl.events.onChange, e.target.value)}
            />
          )}
          {state?.error && <span className="vbr-error">{state.error}</span>}
        </div>
      );
    }

    case 'NumberBox': {
      const val = state?.value ?? '0';
      return (
        <div key={key} className="vbr-field">
          <label className="vbr-field-label">{ctrl.label}</label>
          <input
            type="number"
            className="vbr-input vbr-number"
            value={val}
            min={ctrl.min}
            max={ctrl.max}
            disabled={!(state?.enabled ?? true)}
            onChange={e => ctrl.events?.onChange && dispatch(ctrl.events.onChange, parseFloat(e.target.value))}
          />
          {state?.error && <span className="vbr-error">{state.error}</span>}
        </div>
      );
    }

    case 'CheckBox': {
      const checked = state?.value === 'true';
      return (
        <div key={key} className="vbr-field vbr-checkbox-field">
          <label className="vbr-checkbox-label">
            <input
              type="checkbox"
              checked={checked}
              disabled={!(state?.enabled ?? true)}
              onChange={e => ctrl.events?.onChange && dispatch(ctrl.events.onChange, e.target.checked)}
            />
            <span>{ctrl.label}</span>
          </label>
        </div>
      );
    }

    case 'RadioGroup': {
      const selected = state?.value ?? (ctrl.options?.[0] ?? '');
      return (
        <div key={key} className="vbr-field vbr-radio-group">
          <span className="vbr-field-label">{ctrl.label}</span>
          {ctrl.options?.map((opt, j) => (
            <label key={j} className="vbr-radio-option">
              <input
                type="radio"
                name={ctrl.id}
                value={opt}
                checked={selected === opt}
                onChange={() => ctrl.events?.onChange && dispatch(ctrl.events.onChange, opt)}
              />
              <span>{opt}</span>
            </label>
          ))}
        </div>
      );
    }

    case 'DropDown': {
      const selected = state?.value ?? '';
      return (
        <div key={key} className="vbr-field">
          <label className="vbr-field-label">{ctrl.label}</label>
          <select
            className="vbr-select"
            value={selected}
            disabled={!(state?.enabled ?? true)}
            onChange={e => ctrl.events?.onChange && dispatch(ctrl.events.onChange, e.target.value)}
          >
            {ctrl.options?.map((opt, j) => (
              <option key={j} value={opt}>{opt}</option>
            ))}
          </select>
        </div>
      );
    }

    case 'Button': {
      const btnCls = ['vbr-button', ctrl.style?.toLowerCase()].filter(Boolean).join(' ');
      return (
        <button
          key={key}
          className={btnCls}
          disabled={!(state?.enabled ?? true)}
          onClick={() => ctrl.events?.onClick && dispatch(ctrl.events.onClick)}
        >
          {state?.caption ?? ctrl.text}
        </button>
      );
    }

    case 'Row': {
      const alignCls = `vbr-row vbr-row-${ctrl.align?.toLowerCase() ?? 'left'}`;
      return (
        <div key={key} className={alignCls}>
          {renderControls(ctrl.controls ?? [], getState, dispatch)}
        </div>
      );
    }

    case 'Group':
      return (
        <fieldset key={key} className="vbr-group">
          <legend className="vbr-group-title">{ctrl.title}</legend>
          {renderControls(ctrl.controls ?? [], getState, dispatch)}
        </fieldset>
      );

    case 'ProgressBar': {
      const progress = parseFloat(state?.value ?? '0');
      return (
        <div key={key} className="vbr-field">
          <label className="vbr-field-label">{ctrl.label}</label>
          <progress className="vbr-progress" value={progress} max={1} />
          <span className="vbr-progress-pct">{Math.round(progress * 100)}%</span>
        </div>
      );
    }

    case 'StatusBar':
      return (
        <div key={key} className="vbr-status-bar">
          <span>{state?.value}</span>
        </div>
      );

    case 'Image':
      return (
        <img
          key={key}
          src={ctrl.source}
          className="vbr-image"
          style={{ maxWidth: ctrl.width, maxHeight: ctrl.height }}
          alt=""
        />
      );

    case 'Svg':
      return (
        <img
          key={key}
          src={ctrl.source}
          className="vbr-svg"
          style={{ maxWidth: ctrl.width, maxHeight: ctrl.height }}
          alt=""
        />
      );

    default:
      return null;
  }
}
