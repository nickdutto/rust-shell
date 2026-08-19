use crate::engine::command::Command;
use crate::engine::command::signature::Signature;
use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Table, TableComponent};
use nu_ansi_term::{Color, Style};
use std::fmt::Write;
use std::sync::Arc;

pub fn generate_command_help(command: &Arc<dyn Command + Send + Sync>) -> String {
    let mut buffer = String::new();
    let heading_style = Style::new().fg(Color::Fixed(34));
    let list_head_style = Style::new().fg(Color::LightGreen);

    let _ = writeln!(buffer, "{}:", heading_style.paint("Description"));
    let _ = writeln!(buffer, "  {}\n", command.description());

    let _ = writeln!(buffer, "{}:", heading_style.paint("Info"));
    let _ = writeln!(
        buffer,
        "  {}: {}",
        list_head_style.paint("name"),
        command.name()
    );
    let _ = writeln!(
        buffer,
        "  {}: {}",
        list_head_style.paint("category"),
        command.signature().category
    );
    let _ = writeln!(
        buffer,
        "  {}: {}",
        list_head_style.paint("command_type"),
        command.command_type()
    );

    let _ = writeln!(buffer, "\n{}:", heading_style.paint("Rest Arguments"));
    for line in signature_rest_table(command.signature()).lines() {
        let _ = writeln!(buffer, "  {line}");
    }

    let _ = writeln!(buffer, "\n{}:", heading_style.paint("Positional Arguments"));
    for line in signature_positional_table(command.signature()).lines() {
        let _ = writeln!(buffer, "  {line}");
    }

    let _ = writeln!(buffer, "\n{}:", heading_style.paint("Named Arguments"));
    for line in signature_named_table(command.signature()).lines() {
        let _ = writeln!(buffer, "  {line}");
    }

    buffer
}

pub fn signature_rest_table(signature: Signature) -> Table {
    let mut table = preset_table();

    table.set_header(vec![
        Cell::new("name").fg(comfy_table::Color::Green),
        Cell::new("required").fg(comfy_table::Color::Green),
        Cell::new("shape").fg(comfy_table::Color::Green),
        Cell::new("description").fg(comfy_table::Color::Green),
    ]);

    if let Some(rest_positional) = signature.rest_positional {
        table.add_row(vec![
            rest_positional.name.to_owned(),
            rest_positional.required.to_string(),
            rest_positional.shape.to_string(),
            rest_positional.description.to_owned(),
        ]);
    }

    table
}

pub fn signature_positional_table(signature: Signature) -> Table {
    let mut table = preset_table();

    table.set_header(vec![
        Cell::new("name").fg(comfy_table::Color::Green),
        Cell::new("required").fg(comfy_table::Color::Green),
        Cell::new("shape").fg(comfy_table::Color::Green),
        Cell::new("description").fg(comfy_table::Color::Green),
    ]);

    for positional in signature.positionals {
        table.add_row(vec![
            positional.name.to_owned(),
            positional.required.to_string(),
            positional.shape.to_string(),
            positional.description.to_owned(),
        ]);
    }

    table
}

pub fn signature_named_table(signature: Signature) -> Table {
    let mut table = preset_table();

    table.set_header(vec![
        Cell::new("name").fg(comfy_table::Color::Green),
        Cell::new("short").fg(comfy_table::Color::Green),
        Cell::new("required").fg(comfy_table::Color::Green),
        Cell::new("shape").fg(comfy_table::Color::Green),
        Cell::new("description").fg(comfy_table::Color::Green),
    ]);

    for named in signature.named {
        table.add_row(vec![
            named.name.to_owned(),
            named.short.map_or(" ".to_string(), |c| c.to_string()),
            named.required.to_string(),
            named.shape.to_string(),
            named.description.to_owned(),
        ]);
    }

    table
}

fn preset_table() -> Table {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_style(TableComponent::HeaderLines, '─')
        .set_style(TableComponent::LeftHeaderIntersection, '├')
        .set_style(TableComponent::MiddleHeaderIntersections, '┼')
        .set_style(TableComponent::RightHeaderIntersection, '┤');

    table
}
