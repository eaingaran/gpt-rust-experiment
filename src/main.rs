#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod openai_chat;

use eframe::egui;
use egui::{FontFamily, FontId, TextStyle};
use openai_chat::{ChatMessage, Model, OpenAPI};
use tokio::runtime::{Handle, Runtime};

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        min_window_size: Some(egui::vec2(680.0, 900.0)),
        initial_window_size: Some(egui::vec2(1000.0, 1440.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Rust GPT UI",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

fn get_runtime_handle() -> (Handle, Option<Runtime>) {
    match Handle::try_current() {
        Ok(h) => (h, None),
        Err(_) => {
            let rt = Runtime::new().unwrap();
            (rt.handle().clone(), Some(rt))
        }
    }
}

struct MyApp {
    model: openai_chat::Model,
    api_key: String,
    system_message: String,
    messages: Vec<ChatMessage>,
    message_input: String,
    openai_api: Option<OpenAPI>,
    response_tokens: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            model: openai_chat::Model::Gpt3_5Turbo,
            api_key: "<your-openai-api-key-here>".to_owned(),
            system_message: "You are a friendly assistant.".to_owned(),
            messages: Vec::new(),
            message_input: "Click on 'Start Chat' to get started".to_owned(),
            openai_api: None,
            response_tokens: 10,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use FontFamily::Proportional;
        use TextStyle::*;
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(20.0, Proportional)),
            (Monospace, FontId::new(16.0, Proportional)),
            (Button, FontId::new(16.0, Proportional)),
            (Small, FontId::new(12.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("ChatGPT");
            });
            ui.separator();
            egui::ComboBox::from_label("Select GPT model")
                .selected_text(format!("{:?}", self.model))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.model,
                        Model::Gpt3_5Turbo,
                        Model::Gpt3_5Turbo.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.model,
                        Model::Gpt4,
                        Model::Gpt4.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.model,
                        Model::Gpt4_32k,
                        Model::Gpt4_32k.as_str(),
                    );
                });
            ui.horizontal(|ui| {
                ui.label("System Message : ");
                ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.system_message));
            });
            ui.horizontal(|ui| {
                ui.label("Your API Key :         ");
                ui.add_sized(ui.available_size()*0.8, egui::TextEdit::singleline(&mut self.api_key));
                if ui.button("Start Chat").clicked() {
                    if self.openai_api.is_some() {
                        self.messages.push(ChatMessage { role: "rust".to_owned(), content: "A chat session is in progress. if you like to begin a new session, please end the current session by clicking the 'End Chat' button. Alternatively, you can clear the messages using 'Clear Chat' button".to_owned() });
                    } else if self.api_key == "<your-openai-api-key-here>" {
                        self.messages.push(ChatMessage { role: "rust".to_owned(), content: "Please enter your openai API key and try again".to_owned() });
                    } else {
                        // since these are clones, changing the values in the UI will not affect the chat object
                        self.openai_api = Some(OpenAPI::new(self.api_key.clone(), self.model.clone(), self.system_message.clone()));
                        self.message_input = "Send a message...".to_owned();
                    }
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                let width = ui.available_width();
                ui.add_space(width*0.3);
                if ui.button("End Chat").clicked() {
                    if self.openai_api.is_some() {
                        self.openai_api.as_mut().unwrap().clear_chat();
                        self.messages.clear();
                        self.openai_api = None;
                        self.message_input = "Click on 'Start Chat' to get started".to_owned();
                    } else {
                        self.messages.push(ChatMessage { role: "rust".to_owned(), content: "No active chat session found to terminate. Please start a new session by clicking 'Start Chat'".to_owned() });
                    }
                }
                ui.add_space(width*0.1);
                if ui.button("Clear Chat").clicked() {
                    if self.openai_api.is_some() {
                        self.openai_api.as_mut().unwrap().clear_chat();
                        self.messages.clear();
                        self.message_input = "Send a message...".to_owned();
                    } else {
                        self.messages.push(ChatMessage { role: "rust".to_owned(), content: "No active chat session found to clear. Please start a new session by clicking 'Start Chat'".to_owned() });
                    }
                }
            });
            ui.separator();

            ui.add_space(10.0);
            ui.add(egui::Slider::new(&mut self.response_tokens, 1..=32768).text("Max Tokens"));
            ui.add_space(10.0);

            let mut view_text = String::new();

            for message in self.messages.clone() {
                let role = message.role;
                let content = message.content;
                view_text.push_str(format!("\n{role:<8}: {content:#?}").as_str());
            }

            egui::ScrollArea::vertical().show_rows(ui, 20.0, self.messages.capacity(), |ui, _row_range| {
                for message in self.messages.clone() {
                    let role = message.role;
                    let content = message.content;
                    ui.label(format!("\n{role:<16}: {content:#?}").as_str());
                }
            });

            let (handle, _rt) = get_runtime_handle();

            ui.horizontal(|ui| {
                let _chat_box = ui.add_sized(ui.available_size()*0.8, egui::TextEdit::multiline(&mut self.message_input));
                if ui.button("Send").clicked() /*|| (chat_box.has_focus() && chat_box.ctx.input(|i| {i.key_pressed(egui::Key::Enter)}) && chat_box.ctx.input(|i| {i.keys_down.len() == 1}))*/ {
                    if self.openai_api.is_some() {
                        let message = self.message_input.clone();
                        self.messages.push(ChatMessage { role: "user".to_owned(), content: self.message_input.clone().to_owned() });
                        let model_name = self.model.as_str();
                        self.message_input = format!("fetching result from {model_name}...").to_owned();
                        let response = handle.block_on(self.openai_api.as_mut().unwrap().chat(message, self.response_tokens));
                        self.message_input.clear();
                        self.messages.push(response);
                    } else {
                        self.messages.push(ChatMessage { role: "rust".to_owned(), content: "Chat session is not active. Please start a new session by clicking 'Start Chat'".to_owned() });
                    }
                }
            });
        });
    }
}
