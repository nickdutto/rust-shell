use crate::error::shell_error::ShellError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub enum DataSizeUnit {
    Byte,
    KiB,
    MiB,
    GiB,
    TiB,
    KB,
    MB,
    GB,
    TB,
}

impl Display for DataSizeUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSizeUnit::Byte => write!(f, "B"),
            DataSizeUnit::KiB => write!(f, "KiB"),
            DataSizeUnit::MiB => write!(f, "MiB"),
            DataSizeUnit::GiB => write!(f, "GiB"),
            DataSizeUnit::TiB => write!(f, "TiB"),
            DataSizeUnit::KB => write!(f, "KB"),
            DataSizeUnit::MB => write!(f, "MB"),
            DataSizeUnit::GB => write!(f, "GB"),
            DataSizeUnit::TB => write!(f, "TB"),
        }
    }
}

impl FromStr for DataSizeUnit {
    type Err = ShellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "byte" => Ok(DataSizeUnit::Byte),
            "kib" => Ok(DataSizeUnit::KiB),
            "mib" => Ok(DataSizeUnit::MiB),
            "gib" => Ok(DataSizeUnit::GiB),
            "tib" => Ok(DataSizeUnit::TiB),
            "kb" => Ok(DataSizeUnit::KB),
            "mb" => Ok(DataSizeUnit::MB),
            "gb" => Ok(DataSizeUnit::GB),
            "tb" => Ok(DataSizeUnit::TB),
            _ => Err(ShellError::Generic(format!(
                "`{s}` does not map to a valid DataSizeUnit variant."
            ))),
        }
    }
}

pub fn convert_data_size_unit(bytes: u64, unit: DataSizeUnit) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let bytes_f = bytes as f64;

    match unit {
        DataSizeUnit::Byte => bytes_f,
        DataSizeUnit::KiB => bytes_f / 1024.0,
        DataSizeUnit::MiB => bytes_f / 1024.0f64.powi(2),
        DataSizeUnit::GiB => bytes_f / 1024.0f64.powi(3),
        DataSizeUnit::TiB => bytes_f / 1024.0f64.powi(4),
        DataSizeUnit::KB => bytes_f / 1000.0,
        DataSizeUnit::MB => bytes_f / 1000.0f64.powi(2),
        DataSizeUnit::GB => bytes_f / 1000.0f64.powi(3),
        DataSizeUnit::TB => bytes_f / 1000.0f64.powi(4),
    }
}

pub fn auto_data_size_binary(bytes: u64) -> (f64, DataSizeUnit) {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;
    const TIB: u64 = 1024 * 1024 * 1024 * 1024;

    match bytes {
        b if b >= TIB => (
            convert_data_size_unit(b, DataSizeUnit::TiB),
            DataSizeUnit::TiB,
        ),

        b if b >= GIB => (
            convert_data_size_unit(b, DataSizeUnit::GiB),
            DataSizeUnit::GiB,
        ),

        b if b >= MIB => (
            convert_data_size_unit(b, DataSizeUnit::MiB),
            DataSizeUnit::MiB,
        ),

        b if b >= KIB => (
            convert_data_size_unit(b, DataSizeUnit::KiB),
            DataSizeUnit::KiB,
        ),

        b => (
            convert_data_size_unit(b, DataSizeUnit::Byte),
            DataSizeUnit::Byte,
        ),
    }
}

pub fn auto_data_size_decimal(bytes: u64) -> (f64, DataSizeUnit) {
    const KB: u64 = 1000;
    const MB: u64 = 1000 * 1000;
    const GB: u64 = 1000 * 1000 * 1000;
    const TB: u64 = 1000 * 1000 * 1000 * 1000;

    match bytes {
        b if b >= TB => (
            convert_data_size_unit(b, DataSizeUnit::TB),
            DataSizeUnit::TB,
        ),

        b if b >= GB => (
            convert_data_size_unit(b, DataSizeUnit::GB),
            DataSizeUnit::GB,
        ),

        b if b >= MB => (
            convert_data_size_unit(b, DataSizeUnit::MB),
            DataSizeUnit::MB,
        ),

        b if b >= KB => (
            convert_data_size_unit(b, DataSizeUnit::KB),
            DataSizeUnit::KB,
        ),

        b => (
            convert_data_size_unit(b, DataSizeUnit::Byte),
            DataSizeUnit::Byte,
        ),
    }
}
