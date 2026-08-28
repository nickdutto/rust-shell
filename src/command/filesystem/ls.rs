use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::value::data_size_unit::{DataSizeUnit, auto_data_size_binary};
use crate::value::syntax_shape::SyntaxShape;
use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table, TableComponent,
};
use jiff::Timestamp;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::{DirEntry, ReadDir};
use std::io::Write;
use std::path::PathBuf;

pub struct Ls;

impl Command for Ls {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn description(&self) -> &'static str {
        "List contents of a directory"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::FileSystem)
            .positional(
                "path",
                SyntaxShape::String,
                "The directory path to run in. Defaults to the current directory",
            )
    }

    fn run(
        &self,
        call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let path = call.opt::<PathBuf>(0)?.unwrap_or(std::env::current_dir()?);
        let dir = fs::read_dir(path)?;

        writeln!(io_streams.output, "{}", Self::table_output(dir))?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl Ls {
    fn table_output(dir: ReadDir) -> Table {
        let mut table = build_table();

        let mut entries: Vec<EntryMetadata> =
            dir.flatten().map(|e| dir_entry_metadata(&e)).collect();

        entries.sort_by_key(|e| (e.file_type.priority(), e.file_name.clone()));

        for entry in entries {
            table.add_row([
                entry.file_name,
                entry.file_type.to_string(),
                format!("{:.2} {}", entry.size, entry.data_size_unit),
                entry.modified,
            ]);
        }

        table
    }
}

enum FileType {
    Dir,
    File,
    Symlink,
    Unknown,
}

impl FileType {
    fn priority(&self) -> u8 {
        match self {
            FileType::Dir => 0,
            FileType::File => 1,
            FileType::Symlink => 2,
            FileType::Unknown => 3,
        }
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Dir => write!(f, "dir"),
            FileType::File => write!(f, "file"),
            FileType::Symlink => write!(f, "symlink"),
            FileType::Unknown => write!(f, "unknown"),
        }
    }
}

struct EntryMetadata {
    file_name: String,
    file_type: FileType,
    size: f64,
    data_size_unit: DataSizeUnit,
    modified: String,
}

fn dir_entry_metadata(entry: &DirEntry) -> EntryMetadata {
    let file_type = match entry.file_type() {
        Ok(ft) if ft.is_dir() => FileType::Dir,
        Ok(ft) if ft.is_file() => FileType::File,
        Ok(ft) if ft.is_symlink() => FileType::Symlink,
        _ => FileType::Unknown,
    };

    let (size, data_size_unit, modified) = if let Ok(metadata) = entry.metadata() {
        let modified = if let Ok(st) = metadata.modified() {
            Timestamp::try_from(st)
                .map_or(String::new(), |ts| ts.strftime("%d/%M/%Y %R").to_string())
        } else {
            String::new()
        };

        let (size, unit) = auto_data_size_binary(metadata.len());
        (size, unit, modified)
    } else {
        (0.0, DataSizeUnit::Byte, String::new())
    };

    EntryMetadata {
        file_name: entry.file_name().to_string_lossy().to_string(),
        file_type,
        size,
        data_size_unit,
        modified,
    }
}

fn build_table() -> Table {
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

    table.set_header(vec![
        Cell::new("name")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Center),
        Cell::new("type")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Center),
        Cell::new("size")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Center),
        Cell::new("modified")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold)
            .set_alignment(CellAlignment::Center),
    ]);

    table
}
