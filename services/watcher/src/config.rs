use std::{collections::HashSet, path::PathBuf, time::Duration};

use core_lib::{GlobalState, LuaVM};

use crate::error::WatcherError;

/// Extension Trait to extract config for the current plugin from the LuaVM
pub trait LuaWatcherExt {
    /// Extract the path to watch
    fn watch_dir(&self) -> Result<PathBuf, WatcherError>;
    /// Extract the list of allowed extentions
    fn allowed_extensions(&self) -> Result<HashSet<String>, WatcherError>;
    /// Extract the FS events debounc eduration (in millis)
    fn debounce_duration(&self) -> Result<Duration, WatcherError>;
}

impl LuaWatcherExt for LuaVM {
    fn watch_dir(&self) -> Result<PathBuf, WatcherError> {
        let path = self.get_config_value("watch_dir")?;
        Ok(path)
    }

    fn allowed_extensions(&self) -> Result<HashSet<String>, WatcherError> {
        let allowed_types = self.get_config_value("allowed_extensions")?;
        Ok(allowed_types)
    }

    fn debounce_duration(&self) -> Result<Duration, WatcherError> {
        let millis = self.get_config_value("debounce_duration")?;
        let debounce_duration = Duration::from_millis(millis);
        Ok(debounce_duration)
    }
}

/// Structure to hold configuration extracted from Lua
pub struct Config {
    /// The directory being monitored.
    pub watch_dir: PathBuf,

    /// Set of file extensions that this watcher cares about.
    pub allowed_extensions: HashSet<String>,

    /// Time to wait for file system stability before triggering events.
    pub debounce_duration: Duration,
}

impl Config {
    /// Prepares configuration by setting defaults and extracting values from Lua VM
    pub fn new(state: &GlobalState) -> Result<Config, WatcherError> {
        let lua_vm = &state.lua_vm;

        lua_vm.set_default_config_value("watch_dir", "/default/music/dir")?;
        lua_vm.set_default_config_value("debounce_duration", 2000)?;

        Ok(Config {
            watch_dir: lua_vm.watch_dir()?,
            allowed_extensions: lua_vm.allowed_extensions()?,
            debounce_duration: lua_vm.debounce_duration()?,
        })
    }
}
