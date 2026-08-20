use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::syntax_shape::SyntaxShape;
use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use std::io::Write;
use sysinfo::Disks;

pub struct SysDisk;

impl Command for SysDisk {
    fn name(&self) -> &'static str {
        "sys disk"
    }

    fn description(&self) -> &'static str {
        "System disk information"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::System)
            .named(
                "units",
                SyntaxShape::String,
                "Storage units in bytes (default GB). options: B, KB, MB, GB, TB",
                Some('u'),
            )
    }

    fn run(
        &self,
        call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let storage_units = call
            .opt_named::<String>("units")?
            .unwrap_or(String::from("GB"));

        writeln!(io_streams.output, "{}", Self::disk_table(&storage_units))?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl SysDisk {
    fn disk_table(storage_units: &str) -> Table {
        let mut table = Table::new();

        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("name").add_attribute(Attribute::Bold),
                Cell::new("free_space").add_attribute(Attribute::Bold),
                Cell::new("total_space").add_attribute(Attribute::Bold),
                Cell::new("file_system").add_attribute(Attribute::Bold),
                Cell::new("kind").add_attribute(Attribute::Bold),
                Cell::new("read_only").add_attribute(Attribute::Bold),
                Cell::new("removable").add_attribute(Attribute::Bold),
            ]);

        for disk in &Disks::new_with_refreshed_list() {
            table.add_row([
                disk.name().to_string_lossy().to_string(),
                format!(
                    "{:.2} {storage_units}",
                    convert_bytes(disk.available_space(), storage_units)
                ),
                format!(
                    "{:.2} {storage_units}",
                    convert_bytes(disk.total_space(), storage_units)
                ),
                disk.file_system().to_string_lossy().to_string(),
                disk.kind().to_string(),
                disk.is_read_only().to_string(),
                disk.is_removable().to_string(),
            ]);
        }

        table
    }
}

fn convert_bytes(bytes: u64, storage_unit: &str) -> f64 {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

    // TODO: loss of precision possible
    let bytes_f = bytes as f64;

    match storage_unit {
        "KB" => bytes_f / KB,
        "MB" => bytes_f / MB,
        "GB" => bytes_f / GB,
        "TB" => bytes_f / TB,
        _ => bytes_f,
    }
}
