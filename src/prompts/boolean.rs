use promptuity::{
    prompts::{Confirm, ConfirmFormatter},
    Prompt,
};
use serde_json::Value;

pub struct Boolean {
    prompt: Confirm,
}

impl Boolean {
    pub fn new(message: String) -> Self {
        Self {
            prompt: Confirm::new(message),
        }
    }

    pub fn with_formatter(&mut self, formatter: impl ConfirmFormatter + 'static) -> &mut Self {
        self.prompt.with_formatter(formatter);
        self
    }

    pub fn with_hint(&mut self, hint: impl std::fmt::Display) -> &mut Self {
        self.prompt.with_hint(hint);
        self
    }

    pub fn with_default(&mut self, value: bool) -> &mut Self {
        self.prompt.with_default(value);
        self
    }
}

impl Prompt for Boolean {
    type Output = Value;

    fn setup(&mut self) -> Result<(), promptuity::Error> {
        self.prompt.setup()
    }

    fn handle(
        &mut self,
        code: promptuity::event::KeyCode,
        modifiers: promptuity::event::KeyModifiers,
    ) -> promptuity::PromptState {
        self.prompt.handle(code, modifiers)
    }

    fn submit(&mut self) -> Self::Output {
        Value::Bool(self.prompt.submit())
    }

    fn render(
        &mut self,
        state: &promptuity::PromptState,
    ) -> Result<promptuity::RenderPayload, String> {
        self.prompt.render(state)
    }

    fn validate(&self) -> Result<(), String> {
        self.prompt.validate()
    }
}
