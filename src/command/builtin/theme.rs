use crate::io::stream::IoStreams;
use crate::io::tokenize::Tokens;
use nu_ansi_term::{Color, Style};
use std::fmt::Write as FmtWrite;
use std::io::Write;

pub fn handle_theme(tokens: Tokens, mut io_streams: IoStreams) {
    let mut buffer = String::with_capacity(8192);

    for arg in tokens.arguments.iter() {
        match arg.as_str() {
            "-fg" => {
                for color_idx in 0..256 {
                    write!(
                        buffer,
                        "{}",
                        Style::new()
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
                for color_idx in 0..256 {
                    write!(
                        buffer,
                        "{}",
                        Style::new()
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
                write!(
                    buffer,
                    "{}  {}  {}  {}  {}  {}",
                    Style::new().blink().paint("blink".to_string()),
                    Style::new().bold().paint("bold".to_string()),
                    Style::new().dimmed().paint("dimmed".to_string()),
                    Style::new().italic().paint("italic".to_string()),
                    Style::new()
                        .strikethrough()
                        .paint("strikethrough".to_string()),
                    Style::new().underline().paint("underline".to_string()),
                )
                .ok();
            }
            "-shape" => {
                let style = Style::new().fg(Color::Fixed(33));

                writeln!(buffer, "{} Solid Right Arrow", style.paint("\u{e0b0}"),).ok();
                writeln!(buffer, "{} Line Left Arrow", style.paint("\u{e0b1}"),).ok();
                writeln!(buffer, "{} Solid Left Arrow", style.paint("\u{e0b2}"),).ok();
                writeln!(buffer, "{} Line Left Arrow", style.paint("\u{e0b3}"),).ok();
                writeln!(buffer, "{} Up Triangle", style.paint("\u{25b2}"),).ok();
                writeln!(buffer, "{} Down Triangle", style.paint("\u{25bc}"),).ok();
                writeln!(buffer, "{} Left Triangle", style.paint("\u{25c0}"),).ok();
                writeln!(buffer, "{} Right Triangle", style.paint("\u{25b6}"),).ok();
                writeln!(buffer, "{} Diamond", style.paint("\u{25c6}"),).ok();
                writeln!(buffer, "{} Solid Square", style.paint("\u{25a0}"),).ok();
            }
            _ => continue,
        }
    }

    if !buffer.is_empty() {
        writeln!(io_streams.output, "{}", buffer.trim()).ok();
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
