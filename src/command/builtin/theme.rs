use crate::io::stream::IoStreams;
use crate::parser::tokenize::Tokens;
use crate::shell::config::Config;
use comfy_table::Table;
use comfy_table::presets::NOTHING;
use nu_ansi_term::{Color, Style};
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::sync::Arc;

pub fn handle_theme(tokens: Tokens, config: Arc<Config>, mut io_streams: IoStreams) {
    let mut buffer = String::with_capacity(8192);

    for arg in tokens.arguments.iter() {
        match arg.as_str() {
            "-fg" => {
                let style = Style::new();
                writeln!(
                    buffer,
                    "{}",
                    style.fg(Color::Fixed(39)).paint("Foreground (ANSI 256)")
                )
                .ok();

                for color_idx in 0..256 {
                    write!(
                        buffer,
                        "{}",
                        style
                            .fg(Color::Fixed(color_idx as u8))
                            .paint(format!("{:^5}", color_idx))
                    )
                    .ok();

                    if (color_idx + 1) % 16 == 0 {
                        buffer.push('\n');
                    }
                }
            }
            "-bg" => {
                let style = Style::new();
                writeln!(
                    buffer,
                    "{}",
                    style.fg(Color::Fixed(39)).paint("Background (ANSI 256)")
                )
                .ok();

                for color_idx in 0..256 {
                    write!(
                        buffer,
                        "{}",
                        style
                            .fg(get_contrast_foreground(color_idx as u8))
                            .on(Color::Fixed(color_idx as u8))
                            .paint(format!("{:^5}", color_idx))
                    )
                    .ok();

                    if (color_idx + 1) % 16 == 0 {
                        buffer.push('\n');
                    }
                }
            }
            "-text" => {
                let style = Style::new();
                let mut table = Table::new();
                table
                    .load_preset(NOTHING)
                    .add_row(vec![style.blink().paint("blink")])
                    .add_row(vec![style.bold().paint("bold")])
                    .add_row(vec![style.dimmed().paint("dimmed")])
                    .add_row(vec![style.italic().paint("italic")])
                    .add_row(vec![style.strikethrough().paint("strikethrough")])
                    .add_row(vec![style.underline().paint("underline")]);

                writeln!(buffer, "{}", style.fg(Color::Fixed(39)).paint("Text")).ok();
                writeln!(buffer, "{table}").ok();
            }
            "-shape" => {
                let style = Style::new();
                let shape_style = Style::new().fg(Color::Fixed(39));
                let mut table = Table::new();
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

                writeln!(buffer, "{}", style.fg(Color::Fixed(39)).paint("Shapes")).ok();
                writeln!(buffer, "{table}").ok();
            }
            "-config" => {
                let style = Style::new();
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

                writeln!(buffer, "{}", style.fg(Color::Fixed(39)).paint("Config")).ok();
                writeln!(buffer, "{table}").ok();
            }

            _ => continue,
        }
    }

    if !buffer.is_empty() {
        writeln!(io_streams.output, "{}", buffer.trim_end()).ok();
    }
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

        let luminance = (r as u32 * 21) + (g as u32 * 72) + (b as u32 * 7);

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
