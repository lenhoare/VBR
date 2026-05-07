// Same form as vbr_forms_cursive/examples/settings.rs — different backend.
// Run with: cargo run --example settings -p vbr_forms_egui

use std::sync::{Arc, Mutex};
use vbr_forms_core::*;
use vbr_forms_egui::EguiBackend;

#[derive(Debug, Default)]
struct SettingsData {
    username: String,
    email: String,
    dark_mode: bool,
    theme: String,
    font_size: String,
    save_progress: f32,
    status_message: String,
}

impl SettingsData {
    fn new() -> Self {
        SettingsData {
            username: String::from("alice"),
            email: String::from("alice@example.com"),
            dark_mode: false,
            theme: String::from("Default"),
            font_size: String::from("Medium"),
            save_progress: 0.0,
            status_message: String::from("Ready"),
        }
    }
}

impl FormData for SettingsData {
    fn get(&self, binding: &str) -> Option<FieldValue> {
        match binding {
            "username"      => Some(FieldValue::Text(self.username.clone())),
            "email"         => Some(FieldValue::Text(self.email.clone())),
            "darkMode"      => Some(FieldValue::Bool(self.dark_mode)),
            "theme"         => Some(FieldValue::Text(self.theme.clone())),
            "fontSize"      => Some(FieldValue::Text(self.font_size.clone())),
            "saveProgress"  => Some(FieldValue::Progress(self.save_progress)),
            "statusMessage" => Some(FieldValue::Text(self.status_message.clone())),
            _ => None,
        }
    }

    fn set(&mut self, binding: &str, value: FieldValue) {
        match (binding, value) {
            ("username",      FieldValue::Text(v))     => self.username = v,
            ("email",         FieldValue::Text(v))     => self.email = v,
            ("darkMode",      FieldValue::Bool(v))     => self.dark_mode = v,
            ("theme",         FieldValue::Text(v))     => self.theme = v,
            ("fontSize",      FieldValue::Text(v))     => self.font_size = v,
            ("saveProgress",  FieldValue::Progress(v)) => self.save_progress = v,
            ("statusMessage", FieldValue::Text(v))     => self.status_message = v,
            _ => {}
        }
    }
}

struct SettingsHandlers {
    data: Arc<Mutex<SettingsData>>,
}

impl EventDispatch for SettingsHandlers {
    fn dispatch(&mut self, handler: &str, value: FieldValue) -> Action {
        match handler {
            "EmailChanged" => {
                if let FieldValue::Text(ref email) = value {
                    let mut d = self.data.lock().unwrap();
                    d.status_message = if email.contains('@') {
                        String::from("Email looks valid.")
                    } else {
                        String::from("Email missing @")
                    };
                }
                Action::None
            }
            "DarkModeToggled" => {
                let mut d = self.data.lock().unwrap();
                d.status_message = if d.dark_mode {
                    String::from("Dark mode on.")
                } else {
                    String::from("Dark mode off.")
                };
                Action::None
            }
            "ThemeChanged" => {
                if let FieldValue::Text(ref t) = value {
                    self.data.lock().unwrap().status_message = format!("Theme set to {}.", t);
                }
                Action::None
            }
            "FontSizeChanged" => {
                if let FieldValue::Text(ref s) = value {
                    self.data.lock().unwrap().status_message = format!("Font size: {}.", s);
                }
                Action::None
            }
            "Save" => {
                let mut d = self.data.lock().unwrap();
                d.save_progress = 1.0;
                d.status_message = format!("Saved settings for {}.", d.username);
                Action::None
            }
            "Cancel" => Action::Quit,
            _ => Action::None,
        }
    }
}

fn build_form() -> FormDef {
    FormDef::new("Application Settings")
        .add(Control::Label(LabelDef::new("Application Settings").style(LabelStyle::Bold)))
        .add(Control::Label(LabelDef::new("Configure your preferences below.").style(LabelStyle::Dim)))
        .add(Control::Separator)
        .add(Control::Group(
            GroupDef::new("User")
                .add(Control::TextBox(TextBoxDef::new("Username:", "username")))
                .add(Control::TextBox(
                    TextBoxDef::new("Email:", "email").on_change("EmailChanged"),
                )),
        ))
        .add(Control::Group(
            GroupDef::new("Display")
                .add(Control::CheckBox(
                    CheckBoxDef::new("Dark mode", "darkMode").on_change("DarkModeToggled"),
                ))
                .add(Control::DropDown(
                    DropDownDef::new("Theme:", "theme")
                        .options(vec!["Default", "Minimal", "Colorful"])
                        .on_change("ThemeChanged"),
                ))
                .add(Control::RadioGroup(
                    RadioGroupDef::new("Font size:", "fontSize")
                        .options(vec!["Small", "Medium", "Large"])
                        .on_change("FontSizeChanged"),
                )),
        ))
        .add(Control::ProgressBar(ProgressBarDef::new("Saving...", "saveProgress")))
        .add(Control::Separator)
        .add(Control::Row(
            RowDef::new(RowAlign::SpaceBetween)
                .add(ButtonDef::new("Save", "Save").style(ButtonStyle::Primary))
                .add(ButtonDef::new("Cancel", "Cancel").style(ButtonStyle::Danger)),
        ))
        .add(Control::StatusBar(StatusBarDef::new("statusMessage")))
}

fn main() {
    let data = Arc::new(Mutex::new(SettingsData::new()));
    let handlers = Arc::new(Mutex::new(SettingsHandlers { data: Arc::clone(&data) }));
    EguiBackend::run(
        build_form(),
        data as Arc<Mutex<dyn FormData>>,
        handlers as Arc<Mutex<dyn EventDispatch>>,
    )
    .unwrap();
}
