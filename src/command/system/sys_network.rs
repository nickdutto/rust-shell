use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use std::io::Write;
use sysinfo::Networks;

pub struct SysNetwork;

impl Command for SysNetwork {
    fn name(&self) -> &'static str {
        "sys network"
    }

    fn description(&self) -> &'static str {
        "System network information"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).category(Category::System)
    }

    fn run(
        &self,
        _call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        writeln!(io_streams.output, "{}", Self::network_table())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl SysNetwork {
    fn network_table() -> Table {
        let mut table = Table::new();

        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("interface").add_attribute(Attribute::Bold),
                Cell::new("received").add_attribute(Attribute::Bold),
                Cell::new("transmitted").add_attribute(Attribute::Bold),
                Cell::new("ip_addresses").add_attribute(Attribute::Bold),
            ]);

        for (interface_name, data) in &Networks::new_with_refreshed_list() {
            table.add_row([
                interface_name.clone(),
                data.total_received().to_string(),
                data.total_transmitted().to_string(),
                data.ip_networks()
                    .iter()
                    .map(|i| i.addr.to_string())
                    .collect::<Vec<_>>()
                    .join(" | "),
            ]);
        }

        table
    }
}
