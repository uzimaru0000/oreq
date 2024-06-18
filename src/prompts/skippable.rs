use promptuity::event::{KeyCode, KeyModifiers};
use promptuity::{Prompt, PromptInput, PromptState, RenderPayload};

pub struct Skippable<P>
where
    P: Prompt,
{
    prompt: P,
    skip: bool,
}

impl<P> Skippable<P>
where
    P: Prompt,
{
    pub fn new(prompt: P) -> Self {
        Self {
            prompt,
            skip: false,
        }
    }
}

impl<P> Prompt for Skippable<P>
where
    P: Prompt,
{
    type Output = Option<P::Output>;

    fn setup(&mut self) -> Result<(), promptuity::Error> {
        self.prompt.setup()
    }

    fn handle(&mut self, code: KeyCode, modifiers: KeyModifiers) -> PromptState {
        match (code, modifiers) {
            (KeyCode::Esc, _) => {
                self.skip = true;
                PromptState::Submit
            }
            _ => self.prompt.handle(code, modifiers),
        }
    }

    fn submit(&mut self) -> Self::Output {
        if self.skip {
            None
        } else {
            Some(self.prompt.submit())
        }
    }

    fn render(&mut self, state: &PromptState) -> Result<RenderPayload, String> {
        let prompt = self.prompt.render(state)?;

        match state {
            PromptState::Submit => Ok(RenderPayload::new(prompt.message.clone(), None, None)
                .input(if self.skip {
                    PromptInput::Raw("Skipped".to_owned())
                } else {
                    prompt.input
                })),
            _ => Ok(RenderPayload::new(
                prompt.message.clone(),
                prompt
                    .hint
                    .map(|x| format!("{}, <Esc> to skip", x))
                    .or(Some("<Esc> to skip".to_owned())),
                prompt.placeholder,
            )
            .input(prompt.input)
            .body(prompt.body)),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if !self.skip {
            self.prompt.validate()
        } else {
            Ok(())
        }
    }
}
