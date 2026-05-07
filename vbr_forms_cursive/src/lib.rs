// VBR Forms — cursive TUI backend
//
// Implements FormBackend using the `cursive` crate.
// Layout is top-down vertical flow — no coordinates.
// Row is the only horizontal escape hatch.
//
// State sharing strategy:
//   - Input callbacks capture Arc clones of data/events directly.
//   - AppState stored in cursive user_data carries the refresh target list
//     so that after any event, display-only views (StatusBar, ProgressBar)
//     are updated from the data struct automatically.

use std::sync::{Arc, Mutex};

use cursive::{
    Cursive,
    theme::{Effect, Style},
    utils::markup::StyledString,
    views::{
        Button, Checkbox, Dialog, DummyView, LinearLayout,
        Panel, ResizedView, ScrollView, SelectView, TextView,
        EditView, TextArea,
    },
    traits::{Nameable, Resizable},
};

use vbr_forms_core::{
    Action, Control, EventDispatch, FieldValue,
    FormBackend, FormData, FormDef, LabelStyle, RowAlign,
};

// ---------------------------------------------------------------------------
// AppState — stored in cursive user_data
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum DisplayKind {
    StatusBar,
    ProgressBar { label: String },
}

struct AppState {
    data: Arc<Mutex<dyn FormData>>,
    /// Bindings for views that need refreshing after any event
    refresh_targets: Vec<(String, DisplayKind)>,
    /// Bindings for TextArea widgets — synced into data before every button dispatch
    textarea_bindings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Backend entry point
// ---------------------------------------------------------------------------

pub struct CursiveBackend;

impl FormBackend for CursiveBackend {
    fn run(
        def: FormDef,
        data: Arc<Mutex<dyn FormData>>,
        events: Arc<Mutex<dyn EventDispatch>>,
    ) -> Result<(), String> {
        let mut siv = cursive::default();
        siv.set_autorefresh(false);

        // Collect display-only bindings before building the layout
        let refresh_targets = collect_refresh_targets(&def.controls);
        let textarea_bindings = collect_textarea_bindings(&def.controls);

        let state = AppState {
            data: Arc::clone(&data),
            refresh_targets,
            textarea_bindings,
        };
        siv.set_user_data(state);

        // Build layout from the form definition
        let layout = build_controls(&def.controls, Arc::clone(&data), Arc::clone(&events));

        let scrollable = ScrollView::new(layout).full_height();

        siv.add_layer(
            Dialog::around(scrollable)
                .title(def.title.as_str())
                .full_screen(),
        );

        siv.add_global_callback('q', |s| s.quit());

        siv.run();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Collect refresh targets (StatusBar, ProgressBar bindings)
// ---------------------------------------------------------------------------

fn collect_refresh_targets(controls: &[Control]) -> Vec<(String, DisplayKind)> {
    let mut targets = Vec::new();
    for control in controls {
        match control {
            Control::StatusBar(def) => {
                targets.push((def.binding.clone(), DisplayKind::StatusBar));
            }
            Control::ProgressBar(def) => {
                targets.push((
                    def.binding.clone(),
                    DisplayKind::ProgressBar { label: def.label.clone() },
                ));
            }
            Control::Group(def) => {
                targets.extend(collect_refresh_targets(&def.controls));
            }
            _ => {}
        }
    }
    targets
}

// ---------------------------------------------------------------------------
// Collect multi-line TextArea bindings
// ---------------------------------------------------------------------------

fn collect_textarea_bindings(controls: &[Control]) -> Vec<String> {
    let mut bindings = Vec::new();
    for control in controls {
        match control {
            Control::TextBox(def) if def.multi_line => {
                bindings.push(def.binding.clone());
            }
            Control::Group(def) => {
                bindings.extend(collect_textarea_bindings(&def.controls));
            }
            _ => {}
        }
    }
    bindings
}

/// Flush every TextArea's current content into the FormData struct.
/// Called before any button dispatch so handlers see up-to-date values.
fn sync_textarea_bindings(siv: &mut Cursive) {
    let bindings: Vec<String> = match siv.user_data::<AppState>() {
        Some(s) => s.textarea_bindings.clone(),
        None => return,
    };
    for binding in bindings {
        if let Some(content) = siv.call_on_name(&binding, |ta: &mut TextArea| {
            ta.get_content().to_string()
        }) {
            if let Some(state) = siv.user_data::<AppState>() {
                state.data.lock().unwrap().set(&binding, FieldValue::Text(content));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Refresh display-only views after any event
// ---------------------------------------------------------------------------

fn refresh_display_views(siv: &mut Cursive) {
    let (targets, values): (Vec<_>, Vec<_>) = {
        let state = match siv.user_data::<AppState>() {
            Some(s) => s,
            None => return,
        };
        let data = state.data.lock().unwrap();
        state
            .refresh_targets
            .iter()
            .map(|(binding, kind)| {
                let value = data.get(binding);
                ((binding.clone(), kind.clone()), value)
            })
            .unzip()
    };

    for ((binding, kind), value) in targets.into_iter().zip(values.into_iter()) {
        match kind {
            DisplayKind::StatusBar => {
                if let Some(FieldValue::Text(t)) = value {
                    siv.call_on_name(&binding, move |v: &mut TextView| {
                        v.set_content(t.clone());
                    });
                }
            }
            DisplayKind::ProgressBar { label } => {
                let p = match value {
                    Some(FieldValue::Progress(p)) => p.clamp(0.0, 1.0),
                    _ => 0.0,
                };
                let filled = (p * 20.0) as usize;
                let bar = format!(
                    "{} [{}{}] {}%",
                    label,
                    "█".repeat(filled),
                    "░".repeat(20 - filled),
                    (p * 100.0) as u32
                );
                siv.call_on_name(&binding, move |v: &mut TextView| {
                    v.set_content(bar.clone());
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Control builder — recursively builds the cursive view tree
// ---------------------------------------------------------------------------

fn build_controls(
    controls: &[Control],
    data: Arc<Mutex<dyn FormData>>,
    events: Arc<Mutex<dyn EventDispatch>>,
) -> LinearLayout {
    let mut layout = LinearLayout::vertical();

    for control in controls {
        match control {
            Control::Label(def) => {
                let text = match def.style {
                    LabelStyle::Bold => StyledString::styled(&def.text, Effect::Bold),
                    LabelStyle::Dim => StyledString::styled(&def.text, Style::from(Effect::Italic)),
                    LabelStyle::Normal => StyledString::plain(&def.text),
                };
                layout.add_child(TextView::new(text));
            }

            Control::Separator => {
                layout.add_child(DummyView.fixed_height(1));
                layout.add_child(
                    TextView::new("─".repeat(60))
                        .full_width(),
                );
                layout.add_child(DummyView.fixed_height(1));
            }

            Control::TextBox(def) => {
                let initial = {
                    let guard = data.lock().unwrap();
                    match guard.get(&def.binding) {
                        Some(FieldValue::Text(t)) => t,
                        _ => String::new(),
                    }
                };

                if def.multi_line {
                    let height = def.view_height.unwrap_or(5) as usize;
                    let area = TextArea::new()
                        .content(initial)
                        .with_name(&def.binding)
                        .full_width()
                        .fixed_height(height);

                    let section = LinearLayout::vertical()
                        .child(TextView::new(&def.label))
                        .child(area);
                    layout.add_child(section);
                } else {
                    let d = Arc::clone(&data);
                    let e = Arc::clone(&events);
                    let binding = def.binding.clone();
                    let on_change = def.on_change.clone();

                    let mut edit = EditView::new().content(initial);
                    if let Some(max) = def.max_length {
                        edit.set_max_content_width(Some(max as usize));
                    }

                    let edit = edit
                        .on_edit(move |siv, text, _| {
                            let value = FieldValue::Text(text.to_string());
                            d.lock().unwrap().set(&binding, value.clone());
                            if let Some(ref h) = on_change {
                                let action = e.lock().unwrap().dispatch(h, value);
                                if matches!(action, Action::Quit) {
                                    siv.quit();
                                    return;
                                }
                            }
                            refresh_display_views(siv);
                        })
                        .with_name(&def.binding)
                        .full_width();

                    let row = LinearLayout::horizontal()
                        .child(label_cell(&def.label))
                        .child(edit);
                    layout.add_child(row);
                }
            }

            Control::NumberBox(def) => {
                let d = Arc::clone(&data);
                let e = Arc::clone(&events);
                let binding = def.binding.clone();
                let on_change = def.on_change.clone();
                let min = def.min;
                let max = def.max;

                let initial = {
                    let guard = data.lock().unwrap();
                    match guard.get(&def.binding) {
                        Some(FieldValue::Number(n)) => format!("{}", n),
                        _ => String::from("0"),
                    }
                };

                let edit = EditView::new()
                    .content(initial)
                    .on_edit(move |siv, text, _| {
                        if let Ok(n) = text.parse::<f64>() {
                            let clamped = match (min, max) {
                                (Some(lo), Some(hi)) => n.clamp(lo, hi),
                                (Some(lo), None) => n.max(lo),
                                (None, Some(hi)) => n.min(hi),
                                (None, None) => n,
                            };
                            let value = FieldValue::Number(clamped);
                            d.lock().unwrap().set(&binding, value.clone());
                            if let Some(ref h) = on_change {
                                let action = e.lock().unwrap().dispatch(h, value);
                                if matches!(action, Action::Quit) {
                                    siv.quit();
                                    return;
                                }
                            }
                            refresh_display_views(siv);
                        }
                    })
                    .with_name(&def.binding)
                    .fixed_width(15);

                let row = LinearLayout::horizontal()
                    .child(label_cell(&def.label))
                    .child(edit);

                layout.add_child(row);
            }

            Control::CheckBox(def) => {
                let d = Arc::clone(&data);
                let e = Arc::clone(&events);
                let binding = def.binding.clone();
                let on_change = def.on_change.clone();

                let initial = {
                    let guard = data.lock().unwrap();
                    matches!(guard.get(&def.binding), Some(FieldValue::Bool(true)))
                };

                let mut cb = Checkbox::new();
                if initial {
                    cb.check();
                }
                let cb = cb
                    .on_change(move |siv, checked| {
                        let value = FieldValue::Bool(checked);
                        d.lock().unwrap().set(&binding, value.clone());
                        if let Some(ref h) = on_change {
                            let action = e.lock().unwrap().dispatch(h, value);
                            if matches!(action, Action::Quit) {
                                siv.quit();
                                return;
                            }
                        }
                        refresh_display_views(siv);
                    })
                    .with_name(&def.binding);

                let row = LinearLayout::horizontal()
                    .child(label_cell(&def.label))
                    .child(cb);

                layout.add_child(row);
            }

            Control::RadioGroup(def) => {
                let d = Arc::clone(&data);
                let e = Arc::clone(&events);
                let binding = def.binding.clone();
                let on_change = def.on_change.clone();

                let initial = {
                    let guard = data.lock().unwrap();
                    match guard.get(&def.binding) {
                        Some(FieldValue::Text(t)) => t,
                        _ => def.options.first().cloned().unwrap_or_default(),
                    }
                };

                let mut select: SelectView<String> = SelectView::new();
                for opt in &def.options {
                    select.add_item_str(opt);
                }
                // Set initial selection
                if let Some(idx) = def.options.iter().position(|o| *o == initial) {
                    select.set_selection(idx);
                }

                let select = select
                    .on_submit(move |siv, item: &String| {
                        let value = FieldValue::Text(item.clone());
                        d.lock().unwrap().set(&binding, value.clone());
                        if let Some(ref h) = on_change {
                            let action = e.lock().unwrap().dispatch(h, value);
                            if matches!(action, Action::Quit) {
                                siv.quit();
                                return;
                            }
                        }
                        refresh_display_views(siv);
                    })
                    .with_name(&def.binding);

                let section = LinearLayout::vertical()
                    .child(TextView::new(
                        StyledString::styled(&def.label, Effect::Bold),
                    ))
                    .child(select);

                layout.add_child(section);
            }

            Control::DropDown(def) => {
                let d = Arc::clone(&data);
                let e = Arc::clone(&events);
                let binding = def.binding.clone();
                let on_change = def.on_change.clone();

                let initial = {
                    let guard = data.lock().unwrap();
                    match guard.get(&def.binding) {
                        Some(FieldValue::Text(t)) => t,
                        _ => def.options.first().cloned().unwrap_or_default(),
                    }
                };

                let mut select: SelectView<String> = SelectView::new().popup();
                for opt in &def.options {
                    select.add_item_str(opt);
                }
                if let Some(idx) = def.options.iter().position(|o| *o == initial) {
                    select.set_selection(idx);
                }

                let select = select
                    .on_submit(move |siv, item: &String| {
                        let value = FieldValue::Text(item.clone());
                        d.lock().unwrap().set(&binding, value.clone());
                        if let Some(ref h) = on_change {
                            let action = e.lock().unwrap().dispatch(h, value);
                            if matches!(action, Action::Quit) {
                                siv.quit();
                                return;
                            }
                        }
                        refresh_display_views(siv);
                    })
                    .with_name(&def.binding);

                let row = LinearLayout::horizontal()
                    .child(label_cell(&def.label))
                    .child(select);

                layout.add_child(row);
            }

            Control::Button(def) => {
                let e = Arc::clone(&events);
                let handler = def.on_click.clone();

                let btn = Button::new(&def.text, move |siv| {
                    sync_textarea_bindings(siv);
                    let action = e.lock().unwrap().dispatch(&handler, FieldValue::Text(String::new()));
                    if matches!(action, Action::Quit) {
                        siv.quit();
                        return;
                    }
                    refresh_display_views(siv);
                });

                layout.add_child(btn);
            }

            Control::Row(def) => {
                let mut row = LinearLayout::horizontal();

                let n = def.buttons.len();
                for (i, btn_def) in def.buttons.iter().enumerate() {
                    let e = Arc::clone(&events);
                    let handler = btn_def.on_click.clone();

                    let btn = Button::new(&btn_def.text, move |siv| {
                        sync_textarea_bindings(siv);
                        let action = e.lock().unwrap().dispatch(&handler, FieldValue::Text(String::new()));
                        if matches!(action, Action::Quit) {
                            siv.quit();
                            return;
                        }
                        refresh_display_views(siv);
                    });

                    match def.align {
                        RowAlign::SpaceBetween => {
                            row.add_child(btn);
                            if i < n - 1 {
                                row.add_child(DummyView.full_width());
                            }
                        }
                        RowAlign::Right => {
                            if i == 0 {
                                row.add_child(DummyView.full_width());
                            }
                            row.add_child(btn);
                        }
                        RowAlign::Center => {
                            if i == 0 {
                                row.add_child(DummyView.full_width());
                            }
                            row.add_child(btn);
                            if i == n - 1 {
                                row.add_child(DummyView.full_width());
                            }
                        }
                        RowAlign::Left => {
                            row.add_child(btn);
                        }
                    }
                }

                layout.add_child(DummyView.fixed_height(1));
                layout.add_child(row);
            }

            Control::Group(def) => {
                let inner = build_controls(&def.controls, Arc::clone(&data), Arc::clone(&events));
                layout.add_child(
                    Panel::new(inner).title(def.title.as_str()),
                );
            }

            Control::ProgressBar(def) => {
                let initial_text = {
                    let guard = data.lock().unwrap();
                    match guard.get(&def.binding) {
                        Some(FieldValue::Progress(p)) => {
                            let p = p.clamp(0.0, 1.0);
                            let filled = (p * 20.0) as usize;
                            format!(
                                "{} [{}{}] {}%",
                                def.label,
                                "█".repeat(filled),
                                "░".repeat(20 - filled),
                                (p * 100.0) as u32
                            )
                        }
                        _ => format!("{} [{}] 0%", def.label, "░".repeat(20)),
                    }
                };
                layout.add_child(
                    TextView::new(initial_text)
                        .with_name(&def.binding)
                        .full_width(),
                );
            }

            Control::StatusBar(def) => {
                let initial = {
                    let guard = data.lock().unwrap();
                    match guard.get(&def.binding) {
                        Some(FieldValue::Text(t)) => t,
                        _ => String::new(),
                    }
                };
                layout.add_child(DummyView.fixed_height(1));
                layout.add_child(
                    TextView::new(StyledString::styled(initial, Effect::Bold))
                        .with_name(&def.binding)
                        .full_width(),
                );
            }
        }
    }

    layout
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Fixed-width label cell used to align form fields
fn label_cell(label: &str) -> ResizedView<TextView> {
    TextView::new(format!("{:<22}", label)).fixed_width(22)
}
