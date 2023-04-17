use derivative::Derivative;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
struct CompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[derivative(Default(value = "Some(0.5)"))]
    temperature: Option<f32>,
    top_p: Option<f32>,
    #[derivative(Default(value = "Some(1)"))]
    n: Option<usize>,
    #[derivative(Default(value = "Some(false)"))]
    stream: Option<bool>,
    stop: Option<Vec<String>>,
    #[derivative(Default(value = "Some(32768)"))]
    max_tokens: Option<usize>,
    #[derivative(Default(value = "Some(0.0)"))]
    presence_penalty: Option<f32>,
    #[derivative(Default(value = "Some(0.0)"))]
    frequency_penalty: Option<f32>,
    #[derivative(Default(value = "Some(HashMap::new())"))]
    logit_bias: Option<HashMap<String, i32>>,
    #[derivative(Default(value = "Some(\"rust\".to_string())"))]
    user: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompletionResponse {
    id: String,
    object: String,
    created: u128,
    model: String,
    usage: Usage,
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: ChatMessage,
    index: usize,
    logprobs: Option<HashMap<String, Vec<f32>>>,
    finish_reason: Option<String>,
    usage: Option<Usage>,
}

// https://platform.openai.com/docs/models/overview
#[derive(Debug, PartialEq, Clone)]
#[allow(non_camel_case_types, dead_code)]
pub(crate) enum Model {
    Gpt3_5Turbo,
    Gpt3_5Turbo0301,
    TextDavinci003,
    TextDavinci002,
    CodeDavinci002,
    Gpt4,
    Gpt4_0314,
    Gpt4_32k,
    Gpt4_32k_0314,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::Gpt3_5Turbo => "gpt-3.5-turbo",
            Model::Gpt3_5Turbo0301 => "gpt-3.5-turbo-0301",
            Model::TextDavinci003 => "text-davinci-003",
            Model::TextDavinci002 => "text-davinci-002",
            Model::CodeDavinci002 => "code-davinci-002",
            Model::Gpt4 => "gpt-4",
            Model::Gpt4_0314 => "gpt-4-0314",
            Model::Gpt4_32k => "gpt-4-32k",
            Model::Gpt4_32k_0314 => "gpt-4-32k-0314",
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Model::Gpt3_5Turbo => "gpt-3.5-turbo".to_string(),
            Model::Gpt3_5Turbo0301 => "gpt-3.5-turbo-0301".to_string(),
            Model::TextDavinci003 => "text-davinci-003".to_string(),
            Model::TextDavinci002 => "text-davinci-002".to_string(),
            Model::CodeDavinci002 => "code-davinci-002".to_string(),
            Model::Gpt4 => "gpt-4".to_string(),
            Model::Gpt4_0314 => "gpt-4-0314".to_string(),
            Model::Gpt4_32k => "gpt-4-32k".to_string(),
            Model::Gpt4_32k_0314 => "gpt-4-32k-0314".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OpenAPI {
    api_key: String,
    model: Model,
    system_prompt: String,
    pub messages: Vec<ChatMessage>,
}

impl OpenAPI {
    pub fn new(api_key: String, model: Model, system_prompt: String) -> OpenAPI {
        let messages = vec![ChatMessage {
            role: "system".to_owned(),
            content: system_prompt.to_owned(),
        }];
        OpenAPI {
            api_key,
            model,
            system_prompt,
            messages,
        }
    }

    pub async fn chat(&mut self, user_prompt: String, max_tokens: usize) -> ChatMessage {
        let url = format!("https://api.openai.com/v1/chat/completions");

        let message = ChatMessage {
            role: "user".to_owned(),
            content: user_prompt.to_owned(),
        };
        self.messages.push(message);

        let request = CompletionRequest {
            model: self.model.to_string(),
            messages: self.messages.clone(),
            temperature: Some(0.5),
            user: Some("rust-agent".to_string()),
            max_tokens: Some(max_tokens),
            ..Default::default()
        };

        let client = reqwest::Client::new();

        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .unwrap()
            .json::<CompletionResponse>()
            .await
            .unwrap();

        if response.choices.len() > 0 {
            self.messages.push(response.choices[0].message.clone());
            return response.choices[0].message.clone();
        } else {
            return ChatMessage {
                role: "rust".to_owned(),
                content: "Soemthing went wrong. Did not get any response for your prompt."
                    .to_owned(),
            };
        }
    }

    pub fn clear_chat(&mut self) -> () {
        let messages = vec![ChatMessage {
            role: "system".to_owned(),
            content: self.system_prompt.to_owned(),
        }];

        self.messages = messages;
    }
}
