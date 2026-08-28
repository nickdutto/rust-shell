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
