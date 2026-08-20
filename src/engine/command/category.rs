use std::fmt::{Display, Formatter};

pub enum Category {
    Core,
    Date,
    Debug,
    Empty,
    FileSystem,
    Help,
    Network,
    Process,
    Shell,
    System,
    Ui,
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Core => write!(f, "core"),
            Category::Date => write!(f, "date"),
            Category::Debug => write!(f, "debug"),
            Category::Empty => write!(f, "empty"),
            Category::FileSystem => write!(f, "filesystem"),
            Category::Help => write!(f, "help"),
            Category::Network => write!(f, "network"),
            Category::Process => write!(f, "process"),
            Category::Shell => write!(f, "shell"),
            Category::System => write!(f, "system"),
            Category::Ui => write!(f, "ui"),
        }
    }
}
