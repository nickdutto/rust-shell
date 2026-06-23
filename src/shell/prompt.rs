use crate::shell::config::Config;
use nu_ansi_term::{Color, Style};
use reedline::DefaultPrompt;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch};
use std::borrow::Cow;

pub struct ShellPrompt {
    config: Config,
}

impl ShellPrompt {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl Prompt for ShellPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        let prompt = Style::new()
            .fg(Color::Fixed(self.config.theme.colors.prompt_left))
            .paint("$")
            .to_string();

        Cow::Owned(prompt)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        let prompt = Style::new()
            .fg(Color::Fixed(self.config.theme.colors.prompt_right))
            .paint("")
            .to_string();

        Cow::Owned(prompt)
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Owned(
            DefaultPrompt::default()
                .render_prompt_indicator(prompt_mode)
                .into_owned(),
        )
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Owned(
            DefaultPrompt::default()
                .render_prompt_multiline_indicator()
                .into_owned(),
        )
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Owned(
            DefaultPrompt::default()
                .render_prompt_history_search_indicator(history_search)
                .into_owned(),
        )
    }
}
