use crate::config::Config;
use crate::engine::command_registry::CommandRegistry;
use crate::engine::signals::Signals;
use crate::shell::shell_state::ShellState;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct EngineState {
    pub config: Arc<Config>,
    pub command_registry: Arc<CommandRegistry>,
    pub shell_state: Arc<RwLock<ShellState>>,
    pub signals: Signals,
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineState {
    pub fn new() -> Self {
        let config = Config::load_or_default();
        let command_registry = CommandRegistry::new().register_builtins();
        let shell_state = ShellState::from_config(&config);

        Self {
            config: Arc::new(config),
            command_registry: Arc::new(command_registry),
            shell_state: Arc::new(RwLock::new(shell_state)),
            signals: Signals::new(),
        }
    }
}
