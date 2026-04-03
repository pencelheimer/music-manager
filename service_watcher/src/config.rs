use std::{collections::HashSet, path::PathBuf};

use core_lib::LuaVM;

use crate::error::WatcherError;

/// Extantion Trait to extract config for the current plugin from the LuaVM
pub trait LuaWatcherExt {
    fn watch_dir(&self) -> Result<PathBuf, WatcherError>;
    fn allowed_extensions(&self) -> Result<HashSet<String>, WatcherError>;
}

impl LuaWatcherExt for LuaVM {
    /// Extract the path to watch
    fn watch_dir(&self) -> Result<PathBuf, WatcherError> {
        let path = self.get_config_value("watch_dir")?;
        Ok(path)
    }

    fn allowed_extensions(&self) -> Result<HashSet<String>, WatcherError> {
        let allowed_types = self.get_config_value("allowed_extensions")?;
        Ok(allowed_types)
    }
}
