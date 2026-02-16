use common::config::{AiProvider, AppConfig};
use iced::Element;
use iced::widget::{column, pick_list, row, slider, text, text_input, toggler};

const PROVIDERS: &[&str] = &["OpenAI", "Anthropic", "Ollama", "Custom"];

#[derive(Debug, Clone)]
pub struct State {
    pub enabled: bool,
    pub provider: AiProvider,
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
    pub personality_name: String,
    pub traits_text: String,
    pub custom_prompt: String,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleEnabled(bool),
    ProviderSelected(String),
    ApiKeyChanged(String),
    ModelChanged(String),
    EndpointChanged(String),
    PersonalityNameChanged(String),
    TraitsChanged(String),
    CustomPromptChanged(String),
    TemperatureChanged(f32),
}

fn provider_label(provider: &AiProvider) -> &'static str {
    match provider {
        AiProvider::OpenAi => "OpenAI",
        AiProvider::Anthropic => "Anthropic",
        AiProvider::Ollama => "Ollama",
        AiProvider::Custom => "Custom",
    }
}

impl State {
    pub fn from_config(config: &AppConfig) -> Self {
        let ai = &config.ai;
        Self {
            enabled: ai.enabled,
            provider: ai.provider.clone(),
            api_key: ai.api_key.clone().unwrap_or_default(),
            model: ai.model.clone(),
            endpoint: ai.endpoint.clone().unwrap_or_default(),
            personality_name: ai.personality.name.clone(),
            traits_text: ai.personality.traits.join(", "),
            custom_prompt: ai.personality.custom_prompt.clone().unwrap_or_default(),
            temperature: ai.temperature,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleEnabled(v) => self.enabled = v,
            Message::ProviderSelected(p) => {
                self.provider = match p.as_str() {
                    "Anthropic" => AiProvider::Anthropic,
                    "Ollama" => AiProvider::Ollama,
                    "Custom" => AiProvider::Custom,
                    _ => AiProvider::OpenAi,
                };
            }
            Message::ApiKeyChanged(v) => self.api_key = v,
            Message::ModelChanged(v) => self.model = v,
            Message::EndpointChanged(v) => self.endpoint = v,
            Message::PersonalityNameChanged(v) => self.personality_name = v,
            Message::TraitsChanged(v) => self.traits_text = v,
            Message::CustomPromptChanged(v) => self.custom_prompt = v,
            Message::TemperatureChanged(v) => self.temperature = v,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let providers: Vec<String> = PROVIDERS.iter().map(|s| (*s).to_string()).collect();
        let selected = Some(provider_label(&self.provider).to_string());

        column![
            text("AI Settings").size(24),
            toggler(self.enabled)
                .label("Enable AI")
                .on_toggle(Message::ToggleEnabled),
            row![
                text("Provider"),
                pick_list(providers, selected, Message::ProviderSelected),
            ]
            .spacing(10),
            row![
                text("API Key"),
                text_input("Enter API key", &self.api_key)
                    .on_input(Message::ApiKeyChanged)
                    .secure(true),
            ]
            .spacing(10),
            row![
                text("Model"),
                text_input("gpt-4", &self.model).on_input(Message::ModelChanged),
            ]
            .spacing(10),
            row![
                text("Endpoint"),
                text_input("https://api.openai.com/v1", &self.endpoint)
                    .on_input(Message::EndpointChanged),
            ]
            .spacing(10),
            text("Personality").size(18),
            row![
                text("Name"),
                text_input("Pet name", &self.personality_name)
                    .on_input(Message::PersonalityNameChanged),
            ]
            .spacing(10),
            row![
                text("Traits"),
                text_input("friendly, curious, playful", &self.traits_text)
                    .on_input(Message::TraitsChanged),
            ]
            .spacing(10),
            row![
                text("Custom Prompt"),
                text_input("Optional custom prompt", &self.custom_prompt)
                    .on_input(Message::CustomPromptChanged),
            ]
            .spacing(10),
            row![
                text("Temperature"),
                slider(0.0..=2.0, self.temperature, Message::TemperatureChanged).step(0.1),
                text(format!("{:.1}", self.temperature)),
            ]
            .spacing(10),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }

    pub fn apply_to(&self, config: &mut AppConfig) {
        config.ai.enabled = self.enabled;
        config.ai.provider = self.provider.clone();
        config.ai.api_key = if self.api_key.is_empty() {
            None
        } else {
            Some(self.api_key.clone())
        };
        config.ai.model.clone_from(&self.model);
        config.ai.endpoint = if self.endpoint.is_empty() {
            None
        } else {
            Some(self.endpoint.clone())
        };
        config
            .ai
            .personality
            .name
            .clone_from(&self.personality_name);
        config.ai.personality.traits = self
            .traits_text
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        config.ai.personality.custom_prompt = if self.custom_prompt.is_empty() {
            None
        } else {
            Some(self.custom_prompt.clone())
        };
        config.ai.temperature = self.temperature;
    }
}
