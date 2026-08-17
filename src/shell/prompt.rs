use crate::config::Config;
use crate::config::prompt::{PromptMode, PromptSegment};
use crate::engine::engine_state::EngineState;
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

#[derive(Clone)]
pub struct ShellPrompt {
    config: Arc<Config>,
    shell_state: Arc<RwLock<ShellState>>,
    transient: bool,
    prompt_time: Option<Zoned>,
}

impl ShellPrompt {
    pub fn new(config: Arc<Config>, shell_state: Arc<RwLock<ShellState>>) -> Self {
        Self {
            config,
            shell_state,
            transient: false,
            prompt_time: None,
        }
    }

    pub fn from_engine_state(engine_state: &EngineState) -> Self {
        Self::new(
            Arc::clone(&engine_state.config),
            Arc::clone(&engine_state.shell_state),
        )
    }

    pub fn set_transient(&mut self, transient: bool) {
        self.transient = transient;
    }

    pub fn refresh_time(&mut self) {
        self.prompt_time = Some(Zoned::now());
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
            PromptMode::CurrentDirectory => {
                let os_str = match segment.full_directory_path.unwrap_or(false) {
                    true => Some(shell_state.current_directory.as_os_str()),
                    false => shell_state.current_directory.file_name(),
                };

                os_str.map_or_else(
                    || Cow::Owned(shell_state.current_directory.display().to_string()),
                    |os_str| os_str.to_string_lossy(),
                )
            }
            PromptMode::DateTime => Cow::Owned(
                self.prompt_time
                    .as_ref()
                    .unwrap_or(&Zoned::now())
                    .strftime(
                        segment
                            .datetime_format
                            .as_deref()
                            .unwrap_or("%d/%m/%Y %H:%M:%S"),
                    )
                    .to_string(),
            ),
        }
    }

    fn get_icon_slice<'a>(icon_unicode: &str, buffer: &'a mut [u8; 4]) -> Option<&'a str> {
        let raw = u32::from_str_radix(icon_unicode, 16).ok()?;
        let ch = char::from_u32(raw)?;

        Some(ch.encode_utf8(buffer))
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

        let style = match segment.bold {
            true => Style::new().fg(fg).on(bg).bold(),
            false => Style::new().fg(fg).on(bg),
        };

        write!(buffer, "{}", style.paint(" ")).ok();

        if self.config.theme.enable_icons
            && let Some(icon_unicode) = &segment.icon_unicode
        {
            let mut icon_buffer = [0u8; 4];
            if let Some(icon) = Self::get_icon_slice(icon_unicode, &mut icon_buffer) {
                write!(buffer, "{}{}", style.paint(icon), style.paint(" ")).ok();
            }
        }

        write!(buffer, "{}", style.paint(&*prompt_value)).ok();
        write!(buffer, "{}", style.paint(" ")).ok();

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
            self.render_segment(&shell_state_guard, &mut buffer, segment, side, is_edge);
        }

        buffer
    }
}

impl Prompt for ShellPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Owned(self.render_segments(PromptSide::Left))
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        match self.transient {
            true => Cow::Borrowed(""),
            false => Cow::Owned(self.render_segments(PromptSide::Right)),
        }
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'_, str> {
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => Cow::Borrowed(" "),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal | PromptViMode::Visual => Cow::Borrowed(">"),
                PromptViMode::Insert => Cow::Borrowed(": "),
            },
            PromptEditMode::Custom(_) => Cow::Borrowed(" "),
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

pub fn make_transient_prompt(prompt: &ShellPrompt) -> Box<dyn Prompt> {
    let mut prompt = prompt.clone();
    prompt.set_transient(true);
    Box::new(prompt)
}
