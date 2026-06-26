use crate::shell::config::{Config, DEFAULT_DATETIME_FORMAT, PromptMode, PromptSegment};
use crate::shell::shell_state::ShellState;
use jiff::Zoned;
use nu_ansi_term::{Color, Style};
use reedline::{DefaultPrompt, PromptViMode};
use reedline::{Prompt, PromptEditMode, PromptHistorySearch};
use std::borrow::Cow;
use std::fmt::Write;
use std::sync::{Arc, RwLock};

#[derive(PartialEq, Copy, Clone)]
enum PromptSide {
    Left,
    Right,
}

pub struct ShellPrompt {
    config: Config,
    shell_state: Arc<RwLock<ShellState>>,
}

impl ShellPrompt {
    pub fn new(config: Config, shell_state: Arc<RwLock<ShellState>>) -> Self {
        Self {
            config,
            shell_state,
        }
    }

    fn get_prompt_mode_value<'a>(
        &self,
        shell_state: &'a ShellState,
        segment: &'a PromptSegment,
    ) -> Cow<'a, str> {
        match segment.mode {
            PromptMode::Empty => Cow::Borrowed(""),
            PromptMode::Username => Cow::Borrowed(&shell_state.username),
            PromptMode::Basic => Cow::Borrowed(segment.basic_value.as_deref().unwrap_or("")),
            PromptMode::CurrentDirectory => shell_state.current_directory.to_str().map_or_else(
                || Cow::Owned(shell_state.current_directory.display().to_string()),
                Cow::Borrowed,
            ),
            PromptMode::DateTime => Cow::Owned(
                Zoned::now()
                    .strftime(
                        segment
                            .datetime_format
                            .as_deref()
                            .unwrap_or(DEFAULT_DATETIME_FORMAT),
                    )
                    .to_string(),
            ),
        }
    }

    fn render_segment(
        &self,
        shell_state: &ShellState,
        buffer: &mut String,
        segment: &PromptSegment,
        side: PromptSide,
        is_edge: bool,
    ) {
        let prompt_value = self.get_prompt_mode_value(shell_state, segment);
        let bg = Color::Fixed(segment.background);
        let fg = Color::Fixed(segment.foreground);
        let arrow_symbol = match side {
            PromptSide::Left => "\u{e0b0}",
            PromptSide::Right => "\u{e0b2}",
        };

        if segment.arrow_left {
            let mut style = match segment.gap {
                true => Style::new().fg(Color::Fixed(segment.arrow_left_color)),
                false => Style::new()
                    .fg(bg)
                    .on(Color::Fixed(segment.arrow_left_color)),
            };

            if side == PromptSide::Left {
                style = style.reverse();
            }

            write!(buffer, "{}", style.paint(arrow_symbol)).ok();
        } else if segment.gap && !is_edge {
            buffer.push(' ');
        }

        let segment_style = match segment.bold {
            true => Style::new().fg(fg).on(bg).bold(),
            false => Style::new().fg(fg).on(bg),
        };

        write!(
            buffer,
            "{}{}{}",
            segment_style.paint(" "),
            segment_style.paint(&*prompt_value),
            segment_style.paint(" ")
        )
        .ok();

        if segment.arrow_right {
            let mut style = Style::new().fg(Color::Fixed(segment.arrow_right_color));
            if side == PromptSide::Right {
                style = style.reverse();
            }

            write!(buffer, "{}", style.paint(arrow_symbol)).ok();
        } else if segment.gap && !is_edge {
            buffer.push(' ');
        }
    }

    fn render_segments(&self, side: PromptSide) -> String {
        let mut buffer = String::with_capacity(512);

        let prompt_segments = match side {
            PromptSide::Left => &self.config.prompt.left,
            PromptSide::Right => &self.config.prompt.right,
        };

        let mut iter = prompt_segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.mode != PromptMode::Empty)
            .peekable();

        let shell_state_guard = self.shell_state.read().unwrap();

        while let Some((idx, segment)) = iter.next() {
            let is_edge = idx == 0 || iter.peek().is_none();
            self.render_segment(&shell_state_guard, &mut buffer, &segment, side, is_edge);
        }

        buffer
    }
}

impl Prompt for ShellPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Owned(self.render_segments(PromptSide::Left))
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Owned(self.render_segments(PromptSide::Right))
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'_, str> {
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => Cow::Borrowed(" "),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => Cow::Owned("〉".into()),
                PromptViMode::Insert => Cow::Owned(": ".into()),
            },
            _ => Cow::Borrowed(" "),
        }
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
