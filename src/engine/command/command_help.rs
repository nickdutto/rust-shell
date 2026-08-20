use crate::engine::command::Command;
use crate::engine::command::signature::{NamedArg, Signature};
use crate::engine::engine_state::EngineState;
use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table, TableComponent};
use nu_ansi_term::{Color, Style};
use std::fmt::{Display, Write};
use std::sync::Arc;

pub fn generate_command_help(
    command: &Arc<dyn Command + Send + Sync>,
    engine_state: &EngineState,
) -> String {
    let mut buffer = String::new();

    write_description(&mut buffer, command.description());
    write_info(&mut buffer, command);
    write_subcommands(&mut buffer, command.name(), engine_state);
    write_arguments(&mut buffer, command);

    buffer
}

pub fn write_section_heading(buffer: &mut String, heading: &str) {
    let heading_style = Style::new().fg(Color::Fixed(34));
    _ = writeln!(buffer, "{}:", heading_style.paint(heading));
}

pub fn write_description(buffer: &mut String, description: &str) {
    write_section_heading(buffer, "Description");
    _ = writeln!(buffer, "  {description}\n");
}

pub fn write_info(buffer: &mut String, command: &Arc<dyn Command + Send + Sync>) {
    let list_head_style = Style::new().fg(Color::LightGreen);

    let items: [(&str, &dyn Display); 3] = [
        ("name", &command.name()),
        ("category", &command.signature().category),
        ("command_type", &command.command_type()),
    ];

    write_section_heading(buffer, "Info");

    for (key, value) in items {
        _ = writeln!(buffer, "  {}: {}", list_head_style.paint(key), value);
    }
}

pub fn write_subcommands(buffer: &mut String, command_name: &str, engine_state: &EngineState) {
    let list_head_style = Style::new().fg(Color::LightGreen);

    let mut subcommands = engine_state
        .command_registry
        .commands
        .iter()
        .filter(|(key, _)| key.starts_with(command_name) && *key != command_name)
        .peekable();

    if subcommands.peek().is_none() {
        return;
    }

    write_section_heading(buffer, "\nSubcommands");

    for (name, subcommand) in subcommands {
        _ = writeln!(
            buffer,
            "  {}: {}",
            list_head_style.paint(name),
            subcommand.description()
        );
    }
}

pub fn write_arguments(buffer: &mut String, command: &Arc<dyn Command + Send + Sync>) {
    let signature = command.signature();

    if signature.rest_positional.is_some() {
        write_section_heading(buffer, "\nRest Argument");

        for line in signature_rest_table(&signature).lines() {
            _ = writeln!(buffer, "  {line}");
        }
    }

    if !signature.positionals.is_empty() {
        write_section_heading(buffer, "\nPositional Arguments");

        for line in signature_positional_table(&signature).lines() {
            _ = writeln!(buffer, "  {line}");
        }
    }

    if !signature.named.is_empty() {
        write_section_heading(buffer, "\nNamed Arguments");

        for line in signature_named_table(&signature).lines() {
            _ = writeln!(buffer, "  {line}");
        }
    }
}

pub fn signature_rest_table(signature: &Signature) -> Table {
    let mut table = preset_table();

    table.set_header(vec![
        Cell::new("name").fg(comfy_table::Color::Green),
        Cell::new("required").fg(comfy_table::Color::Green),
        Cell::new("shape").fg(comfy_table::Color::Green),
        Cell::new("description").fg(comfy_table::Color::Green),
    ]);

    if let Some(rest_positional) = &signature.rest_positional {
        table.add_row(vec![
            rest_positional.name.to_owned(),
            rest_positional.required.to_string(),
            rest_positional.shape.to_string(),
            rest_positional.description.to_owned(),
        ]);
    }

    table
}

pub fn signature_positional_table(signature: &Signature) -> Table {
    let mut table = preset_table();

    table.set_header(vec![
        Cell::new("name").fg(comfy_table::Color::Green),
        Cell::new("required").fg(comfy_table::Color::Green),
        Cell::new("shape").fg(comfy_table::Color::Green),
        Cell::new("description").fg(comfy_table::Color::Green),
    ]);

    for positional in &signature.positionals {
        table.add_row(vec![
            positional.name.to_owned(),
            positional.required.to_string(),
            positional.shape.to_string(),
            positional.description.to_owned(),
        ]);
    }

    table
}

pub fn signature_named_table(signature: &Signature) -> Table {
    let mut table = preset_table();

    table.set_header(vec![
        Cell::new("name").fg(comfy_table::Color::Green),
        Cell::new("short").fg(comfy_table::Color::Green),
        Cell::new("required").fg(comfy_table::Color::Green),
        Cell::new("shape").fg(comfy_table::Color::Green),
        Cell::new("description").fg(comfy_table::Color::Green),
    ]);

    let mut add_row = |named: &NamedArg| {
        table.add_row(vec![
            named.name.to_owned(),
            named.short.map_or(" ".to_string(), |c| c.to_string()),
            named.required.to_string(),
            named.shape.to_string(),
            named.description.to_owned(),
        ]);
    };

    let mut help_arg = None;

    for named in &signature.named {
        if named.name == "help" {
            help_arg = Some(named);
        } else {
            add_row(named);
        }
    }

    if let Some(help) = help_arg {
        add_row(help);
    }

    table
}

fn preset_table() -> Table {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_style(TableComponent::HeaderLines, '─')
        .set_style(TableComponent::LeftHeaderIntersection, '├')
        .set_style(TableComponent::MiddleHeaderIntersections, '┼')
        .set_style(TableComponent::RightHeaderIntersection, '┤');

    table
}
