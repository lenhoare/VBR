// VBR Forms — egui native backend
//
// Implements FormBackend using eframe + egui.
// Layout mirrors the cursive backend: top-down vertical flow,
// Group renders as a framed panel, Row handles horizontal button alignment.
//
// Edit-buffer strategy:
//   TextBox and NumberBox values are kept in a local HashMap<binding, String>
//   so egui can manage cursor/selection state without fighting the data layer.
//   The buffer is initialised from FormData at construction and synced back
//   on every change. Display-only controls (StatusBar, ProgressBar) read
//   directly from FormData each frame.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eframe::egui::{self, Color32, RichText, ScrollArea};

use vbr_forms_core::{
    Action, ButtonStyle, Control, EventDispatch, FieldValue,
    FormBackend, FormData, FormDef, LabelStyle, RowAlign,
};

// ---------------------------------------------------------------------------
// Public backend entry point
// ---------------------------------------------------------------------------

pub struct EguiBackend;

impl FormBackend for EguiBackend {
    fn run(
        def: FormDef,
        data: Arc<Mutex<dyn FormData>>,
        events: Arc<Mutex<dyn EventDispatch>>,
    ) -> Result<(), String> {
        let title = def.title.clone();
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(&title)
                .with_inner_size([640.0, 520.0]),
            ..Default::default()
        };
        eframe::run_native(
            &title,
            options,
            Box::new(|_cc| Ok(Box::new(VbrApp::new(def, data, events)))),
        )
        .map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct VbrApp {
    def: FormDef,
    data: Arc<Mutex<dyn FormData>>,
    events: Arc<Mutex<dyn EventDispatch>>,
    buffers: HashMap<String, String>,
    should_quit: bool,
}

impl VbrApp {
    fn new(
        def: FormDef,
        data: Arc<Mutex<dyn FormData>>,
        events: Arc<Mutex<dyn EventDispatch>>,
    ) -> Self {
        let buffers = init_buffers(&def.controls, &data);
        VbrApp { def, data, events, buffers, should_quit: false }
    }
}

/// Walk the control tree and collect (binding, is_number) pairs for editable fields.
fn collect_buffer_bindings(controls: &[Control], out: &mut Vec<(String, bool)>) {
    for ctrl in controls {
        match ctrl {
            Control::TextBox(d)   => out.push((d.binding.clone(), false)),
            Control::NumberBox(d) => out.push((d.binding.clone(), true)),
            Control::Group(d)     => collect_buffer_bindings(&d.controls, out),
            _ => {}
        }
    }
}

fn init_buffers(controls: &[Control], data: &Arc<Mutex<dyn FormData>>) -> HashMap<String, String> {
    let mut pairs = Vec::new();
    collect_buffer_bindings(controls, &mut pairs);
    let guard = data.lock().unwrap();
    pairs.into_iter().map(|(binding, is_number)| {
        let val = if is_number {
            match guard.get(&binding) {
                Some(FieldValue::Number(n)) => format!("{}", n),
                _ => String::from("0"),
            }
        } else {
            match guard.get(&binding) {
                Some(FieldValue::Text(t)) => t,
                _ => String::new(),
            }
        };
        (binding, val)
    }).collect()
}

// ---------------------------------------------------------------------------
// eframe App
// ---------------------------------------------------------------------------

impl eframe::App for VbrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                let quit = render_controls(
                    ui,
                    &self.def.controls,
                    &self.data,
                    &self.events,
                    &mut self.buffers,
                );
                if quit {
                    self.should_quit = true;
                }
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Control renderers
// ---------------------------------------------------------------------------

fn render_controls(
    ui: &mut egui::Ui,
    controls: &[Control],
    data: &Arc<Mutex<dyn FormData>>,
    events: &Arc<Mutex<dyn EventDispatch>>,
    buffers: &mut HashMap<String, String>,
) -> bool {
    for ctrl in controls {
        if render_control(ui, ctrl, data, events, buffers) {
            return true;
        }
    }
    false
}

fn render_control(
    ui: &mut egui::Ui,
    ctrl: &Control,
    data: &Arc<Mutex<dyn FormData>>,
    events: &Arc<Mutex<dyn EventDispatch>>,
    buffers: &mut HashMap<String, String>,
) -> bool {
    match ctrl {
        // --- Label -----------------------------------------------------------
        Control::Label(def) => {
            let text = match def.style {
                LabelStyle::Bold   => RichText::new(&def.text).strong(),
                LabelStyle::Dim    => RichText::new(&def.text).weak(),
                LabelStyle::Normal => RichText::new(&def.text),
            };
            ui.label(text);
            false
        }

        // --- Separator -------------------------------------------------------
        Control::Separator => {
            ui.separator();
            false
        }

        // --- TextBox (single and multi-line) ---------------------------------
        Control::TextBox(def) => {
            let buf = buffers.entry(def.binding.clone()).or_default();

            let resp = if def.multi_line {
                let row_height = ui.text_style_height(&egui::TextStyle::Body);
                let height = row_height * def.view_height.unwrap_or(5) as f32;
                ui.label(&def.label);
                ui.add_sized(
                    [ui.available_width(), height],
                    egui::TextEdit::multiline(buf),
                )
            } else {
                ui.horizontal(|ui| {
                    ui.label(&def.label);
                    ui.add(egui::TextEdit::singleline(buf).desired_width(f32::INFINITY))
                }).inner
            };

            if resp.changed() {
                data.lock().unwrap().set(&def.binding, FieldValue::Text(buf.clone()));
                // OnChange not fired for multi-line (by design)
                if !def.multi_line {
                    if let Some(ref h) = def.on_change {
                        let action = events.lock().unwrap()
                            .dispatch(h, FieldValue::Text(buf.clone()));
                        if matches!(action, Action::Quit) { return true; }
                    }
                }
            }
            false
        }

        // --- NumberBox -------------------------------------------------------
        Control::NumberBox(def) => {
            let buf = buffers.entry(def.binding.clone()).or_default();
            let resp = ui.horizontal(|ui| {
                ui.label(&def.label);
                ui.add(egui::TextEdit::singleline(buf).desired_width(120.0))
            }).inner;

            if resp.changed() {
                if let Ok(n) = buf.parse::<f64>() {
                    let clamped = match (def.min, def.max) {
                        (Some(lo), Some(hi)) => n.clamp(lo, hi),
                        (Some(lo), None)     => n.max(lo),
                        (None, Some(hi))     => n.min(hi),
                        (None, None)         => n,
                    };
                    data.lock().unwrap().set(&def.binding, FieldValue::Number(clamped));
                    if let Some(ref h) = def.on_change {
                        let action = events.lock().unwrap()
                            .dispatch(h, FieldValue::Number(clamped));
                        if matches!(action, Action::Quit) { return true; }
                    }
                }
            }
            false
        }

        // --- CheckBox --------------------------------------------------------
        Control::CheckBox(def) => {
            let mut checked = {
                let guard = data.lock().unwrap();
                matches!(guard.get(&def.binding), Some(FieldValue::Bool(true)))
            };
            let resp = ui.checkbox(&mut checked, &def.label);
            if resp.changed() {
                data.lock().unwrap().set(&def.binding, FieldValue::Bool(checked));
                if let Some(ref h) = def.on_change {
                    let action = events.lock().unwrap()
                        .dispatch(h, FieldValue::Bool(checked));
                    if matches!(action, Action::Quit) { return true; }
                }
            }
            false
        }

        // --- RadioGroup ------------------------------------------------------
        Control::RadioGroup(def) => {
            let mut selected = {
                let guard = data.lock().unwrap();
                match guard.get(&def.binding) {
                    Some(FieldValue::Text(t)) => t,
                    _ => def.options.first().cloned().unwrap_or_default(),
                }
            };
            let prev = selected.clone();
            ui.label(RichText::new(&def.label).strong());
            for opt in &def.options {
                ui.radio_value(&mut selected, opt.clone(), opt.as_str());
            }
            if selected != prev {
                data.lock().unwrap().set(&def.binding, FieldValue::Text(selected.clone()));
                if let Some(ref h) = def.on_change {
                    let action = events.lock().unwrap()
                        .dispatch(h, FieldValue::Text(selected));
                    if matches!(action, Action::Quit) { return true; }
                }
            }
            false
        }

        // --- DropDown --------------------------------------------------------
        Control::DropDown(def) => {
            let mut selected = {
                let guard = data.lock().unwrap();
                match guard.get(&def.binding) {
                    Some(FieldValue::Text(t)) => t,
                    _ => def.options.first().cloned().unwrap_or_default(),
                }
            };
            let prev = selected.clone();
            ui.horizontal(|ui| {
                ui.label(&def.label);
                egui::ComboBox::from_id_salt(&def.binding)
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for opt in &def.options {
                            ui.selectable_value(&mut selected, opt.clone(), opt.as_str());
                        }
                    });
            });
            if selected != prev {
                data.lock().unwrap().set(&def.binding, FieldValue::Text(selected.clone()));
                if let Some(ref h) = def.on_change {
                    let action = events.lock().unwrap()
                        .dispatch(h, FieldValue::Text(selected));
                    if matches!(action, Action::Quit) { return true; }
                }
            }
            false
        }

        // --- Standalone Button -----------------------------------------------
        Control::Button(def) => {
            if styled_button(ui, &def.text, &def.style).clicked() {
                let action = events.lock().unwrap()
                    .dispatch(&def.on_click, FieldValue::Text(String::new()));
                if matches!(action, Action::Quit) { return true; }
            }
            false
        }

        // --- Row (horizontal button bar) -------------------------------------
        Control::Row(def) => {
            ui.add_space(4.0);
            let quit = match def.align {
                RowAlign::Right | RowAlign::SpaceBetween => {
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let mut quit = false;
                            // right_to_left renders in reverse, so we reverse to keep
                            // the VBR declaration order correct visually
                            for btn_def in def.buttons.iter().rev() {
                                if styled_button(ui, &btn_def.text, &btn_def.style).clicked() {
                                    let action = events.lock().unwrap().dispatch(
                                        &btn_def.on_click,
                                        FieldValue::Text(String::new()),
                                    );
                                    if matches!(action, Action::Quit) { quit = true; }
                                }
                            }
                            quit
                        },
                    ).inner
                }
                RowAlign::Center => {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            let mut quit = false;
                            for btn_def in &def.buttons {
                                if styled_button(ui, &btn_def.text, &btn_def.style).clicked() {
                                    let action = events.lock().unwrap().dispatch(
                                        &btn_def.on_click,
                                        FieldValue::Text(String::new()),
                                    );
                                    if matches!(action, Action::Quit) { quit = true; }
                                }
                            }
                            quit
                        }).inner
                    }).inner
                }
                RowAlign::Left => {
                    ui.horizontal(|ui| {
                        let mut quit = false;
                        for btn_def in &def.buttons {
                            if styled_button(ui, &btn_def.text, &btn_def.style).clicked() {
                                let action = events.lock().unwrap().dispatch(
                                    &btn_def.on_click,
                                    FieldValue::Text(String::new()),
                                );
                                if matches!(action, Action::Quit) { quit = true; }
                            }
                        }
                        quit
                    }).inner
                }
            };
            ui.add_space(4.0);
            quit
        }

        // --- Group (framed panel) --------------------------------------------
        Control::Group(def) => {
            egui::Frame::group(ui.style())
                .show(ui, |ui| {
                    ui.label(RichText::new(&def.title).strong());
                    ui.separator();
                    render_controls(ui, &def.controls, data, events, buffers)
                })
                .inner
        }

        // --- ProgressBar -----------------------------------------------------
        Control::ProgressBar(def) => {
            let progress = {
                let guard = data.lock().unwrap();
                match guard.get(&def.binding) {
                    Some(FieldValue::Progress(p)) => p.clamp(0.0, 1.0),
                    _ => 0.0,
                }
            };
            ui.label(&def.label);
            ui.add(egui::ProgressBar::new(progress).show_percentage());
            false
        }

        // --- StatusBar -------------------------------------------------------
        Control::StatusBar(def) => {
            let text = {
                let guard = data.lock().unwrap();
                match guard.get(&def.binding) {
                    Some(FieldValue::Text(t)) => t,
                    _ => String::new(),
                }
            };
            ui.add_space(4.0);
            ui.separator();
            ui.label(RichText::new(text).strong());
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn styled_button(ui: &mut egui::Ui, text: &str, style: &ButtonStyle) -> egui::Response {
    match style {
        ButtonStyle::Primary => ui.add(
            egui::Button::new(RichText::new(text).color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 100, 200)),
        ),
        ButtonStyle::Danger => ui.add(
            egui::Button::new(RichText::new(text).color(Color32::WHITE))
                .fill(Color32::from_rgb(180, 40, 40)),
        ),
        ButtonStyle::Normal => ui.button(text),
    }
}
