use crate::config::Config;
use crate::engine::call::Call;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use comfy_table::Table;
use comfy_table::presets::NOTHING;
use nu_ansi_term::{Color, Style};
use std::fmt::Write as FmtWrite;
use std::io::Write;

pub struct Theme;

impl Command for Theme {
    fn name(&self) -> &'static str {
        "theme"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .switch("fg", "foreground colors", Some('f'))
            .switch("bg", "background colors", Some('b'))
            .switch("text", "text styles", Some('t'))
            .switch("shape", "shape characters", Some('s'))
            .switch("config", "current config theme", Some('c'))
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut buffer = String::with_capacity(8192);

        if call.has_switch("fg") {
            let style = Style::new();
            let _ = writeln!(
                buffer,
                "{}",
                style.fg(Color::Fixed(39)).paint("Foreground (ANSI 256)")
            );

            for color_idx in 0..256 {
                let _ = write!(
                    buffer,
                    "{}",
                    style
                        .fg(Color::Fixed(u8::try_from(color_idx).unwrap_or(0)))
                        .paint(format!("{color_idx:^5}"))
                );

                if (color_idx + 1) % 16 == 0 {
                    buffer.push('\n');
                }
            }
        }

        if call.has_switch("bg") {
            let style = Style::new();
            let _ = writeln!(
                buffer,
                "{}",
                style.fg(Color::Fixed(39)).paint("Background (ANSI 256)")
            );

            for color_idx in 0..256 {
                let _ = write!(
                    buffer,
                    "{}",
                    style
                        .fg(Self::get_contrast_foreground(
                            u8::try_from(color_idx).unwrap_or(0)
                        ))
                        .on(Color::Fixed(u8::try_from(color_idx).unwrap_or(0)))
                        .paint(format!("{color_idx:^5}"))
                );

                if (color_idx + 1) % 16 == 0 {
                    buffer.push('\n');
                }
            }
        }

        if call.has_switch("text") {
            let style = Style::new();
            let table = Self::text_table(style);

            let _ = writeln!(buffer, "{}", style.fg(Color::Fixed(39)).paint("Text"));
            let _ = writeln!(buffer, "{table}");
        }

        if call.has_switch("shape") {
            let style = Style::new();
            let table = Self::shape_table(style);

            let _ = writeln!(buffer, "{}", style.fg(Color::Fixed(39)).paint("Shapes"));
            let _ = writeln!(buffer, "{table}");
        }

        if call.has_switch("config") {
            let style = Style::new();
            let table = Self::config_table(style, &engine_state.config);

            let _ = writeln!(buffer, "{}", style.fg(Color::Fixed(39)).paint("Config"));
            let _ = writeln!(buffer, "{table}");
        }

        if !buffer.is_empty() {
            writeln!(io_streams.output, "{}", buffer.trim_end())?;
        }

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl Theme {
    fn text_table(style: Style) -> Table {
        let mut table = Table::new();

        table
            .load_preset(NOTHING)
            .add_row(vec![style.blink().paint("blink")])
            .add_row(vec![style.bold().paint("bold")])
            .add_row(vec![style.dimmed().paint("dimmed")])
            .add_row(vec![style.italic().paint("italic")])
            .add_row(vec![style.strikethrough().paint("strikethrough")])
            .add_row(vec![style.underline().paint("underline")]);

        table
    }

    fn shape_table(style: Style) -> Table {
        let mut table = Table::new();
        let shape_style = Style::new().fg(Color::Fixed(39));

        table
            .load_preset(NOTHING)
            .add_row(vec![
                style.paint("Solid Right Arrow"),
                shape_style.paint("\u{e0b0}"),
            ])
            .add_row(vec![
                style.paint("Line Left Arrow"),
                shape_style.paint("\u{e0b1}"),
            ])
            .add_row(vec![
                style.paint("Solid Left Arrow"),
                shape_style.paint("\u{e0b2}"),
            ])
            .add_row(vec![
                style.paint("Line Left Arrow"),
                shape_style.paint("\u{e0b3}"),
            ])
            .add_row(vec![
                style.paint("Up Triangle"),
                shape_style.paint("\u{25b2}"),
            ])
            .add_row(vec![
                style.paint("Down Triangle"),
                shape_style.paint("\u{25bc}"),
            ])
            .add_row(vec![
                style.paint("Left Triangle"),
                shape_style.paint("\u{25c0}"),
            ])
            .add_row(vec![
                style.paint("Right Triangle"),
                shape_style.paint("\u{25b6}"),
            ])
            .add_row(vec![style.paint("Diamond"), shape_style.paint("\u{25c6}")])
            .add_row(vec![
                style.paint("Solid Square"),
                shape_style.paint("\u{25a0}"),
            ]);

        table
    }

    fn config_table(style: Style, config: &Config) -> Table {
        let mut table = Table::new();

        table
            .load_preset(NOTHING)
            .add_row(vec![
                style.paint("Input Base"),
                style
                    .fg(if let Some(color) = config.theme.colors.input_base {
                        Color::Fixed(color)
                    } else {
                        Color::Default
                    })
                    .paint("Hello World"),
            ])
            .add_row(vec![
                style.paint("Builtin Command"),
                style
                    .fg(Color::Fixed(config.theme.colors.builtin_command))
                    .paint("echo"),
            ])
            .add_row(vec![
                style.paint("Quote Strings (double)"),
                style
                    .fg(Color::Fixed(config.theme.colors.double_quote_strings))
                    .paint("\"Hello World\""),
            ])
            .add_row(vec![
                style.paint("Quote Strings (single)"),
                style
                    .fg(Color::Fixed(config.theme.colors.single_quote_strings))
                    .paint("'Hello World'"),
            ])
            .add_row(vec![
                style.paint("Variables (valid)"),
                style
                    .fg(Color::Fixed(config.theme.colors.variable))
                    .paint("$TARGET_DIR ${TARGET_DIR}"),
            ])
            .add_row(vec![
                style.paint("Variables (invalid)"),
                style
                    .fg(Color::Fixed(config.theme.colors.variable_invalid))
                    .paint("$1A#_DIR ${1A#_DIR}"),
            ])
            .add_row(vec![
                style.paint("Pipe"),
                style.fg(Color::Fixed(config.theme.colors.pipe)).paint("|"),
            ])
            .add_row(vec![
                style.paint("Redirection (out)"),
                style
                    .fg(Color::Fixed(config.theme.colors.redirection_out))
                    .paint("> 1>"),
            ])
            .add_row(vec![
                style.paint("Redirection (out append)"),
                style
                    .fg(Color::Fixed(config.theme.colors.redirection_out_append))
                    .paint(">> 1>>"),
            ])
            .add_row(vec![
                style.paint("Redirection (error)"),
                style
                    .fg(Color::Fixed(config.theme.colors.redirection_error))
                    .paint("2>"),
            ])
            .add_row(vec![
                style.paint("Redirection (error append)"),
                style
                    .fg(Color::Fixed(config.theme.colors.redirection_error_append))
                    .paint("2>>"),
            ]);

        table
    }

    fn get_contrast_foreground(bg_idx: u8) -> Color {
        let black = 0;
        let white = 15;

        if bg_idx < 16 {
            return match bg_idx {
                3 | 7 | 10 | 11 | 14 | 15 => Color::Fixed(black),
                _ => Color::Fixed(white),
            };
        }

        if bg_idx < 232 {
            let idx = bg_idx - 16;
            let r = idx / 36;
            let g = (idx % 36) / 6;
            let b = idx % 6;

            let luminance = (u32::from(r) * 21) + (u32::from(g) * 72) + (u32::from(b) * 7);

            return if luminance > 250 {
                Color::Fixed(black)
            } else {
                Color::Fixed(white)
            };
        }

        if bg_idx >= 244 {
            Color::Fixed(black)
        } else {
            Color::Fixed(white)
        }
    }
}
