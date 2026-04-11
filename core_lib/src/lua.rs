use std::path::Path;

use mlua::{FromLua, IntoLua, Lua, Table as LuaTable, Value as LuaValue};
use tracing::{debug, instrument};

use crate::error::{LibError, LuaError};

#[derive(Debug)]
pub struct LuaVM(pub Lua);

impl LuaVM {
    pub fn new() -> Result<Self, LibError> {
        debug!("Initializing new Lua VM");
        let lua = Lua::new();

        Self::setup_config_table(&lua)?;

        Ok(Self(lua))
    }

    fn setup_config_table(lua: &Lua) -> Result<(), LuaError> {
        debug!("Injecting base 'config' table into Lua globals");
        let config_table = lua.create_table()?;

        config_table.set("db_url", "/default/music/dir/db.sqlite")?;

        lua.globals().set("config", config_table)?;
        Ok(())
    }

    #[instrument(skip(self), fields(path = %path.as_ref().display()))]
    pub fn load_user_config(&self, path: impl AsRef<Path>) -> Result<(), LuaError> {
        debug!("Executing user config script");

        self.0.load(path.as_ref()).exec()?;

        Ok(())
    }

    // TODO(pencelheimer): support nested tables
    #[instrument(skip_all, fields(key = %key.as_ref()), err)]
    pub fn get_config_value<T: FromLua>(&self, key: impl AsRef<str>) -> Result<T, LuaError> {
        let config_table: LuaTable = self.0.globals().get("config")?;
        let config_value = config_table.get(key.as_ref())?;
        Ok(config_value)
    }

    // TODO(pencelheimer): support nested tables
    #[instrument(skip(self, default_value), fields(key = %key.as_ref()))]
    pub fn set_default_config_value<V: IntoLua>(
        &self,
        key: impl AsRef<str>,
        default_value: V,
    ) -> Result<(), LuaError> {
        let config_table: LuaTable = self.0.globals().get("config")?;
        let key_ref = key.as_ref();

        let current_value: LuaValue = config_table.get(key_ref)?;

        if current_value.is_nil() {
            debug!("Config value is nil, applying default value");
            config_table.set(key_ref, default_value)?;
        }

        Ok(())
    }

    pub fn pipeline_steps(&self) -> Result<Vec<String>, LuaError> {
        self.get_config_value("pipeline_steps")
    }

    pub fn db_url(&self) -> Result<String, LuaError> {
        self.get_config_value("db_url")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docstr::docstr;
    use mlua::Table;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_user_config_overrides_values() {
        let vm = LuaVM::new().unwrap();
        let lua = &vm.0;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            docstr!(
                /// config.watch_dir = '/custom/path'
            )
        )
        .unwrap();

        vm.load_user_config(temp_file.path())
            .expect("Should load config successfully");

        let globals = lua.globals();
        let config: Table = globals.get("config").unwrap();

        let watch_dir: String = config.get("watch_dir").unwrap();

        assert_eq!(watch_dir, "/custom/path");
    }

    #[test]
    fn test_load_user_config_handles_syntax_errors() {
        let vm = LuaVM::new().unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "config.watch_dir = /missing/quotes").unwrap();

        let result = vm.load_user_config(temp_file.path());
        assert!(result.is_err(), "Expected an error due to bad Lua syntax");

        match result.unwrap_err() {
            LuaError::Mlua(_) => {}
        }
    }

    #[test]
    fn test_load_user_config_missing_file() {
        let vm = LuaVM::new().unwrap();

        let result = vm.load_user_config(Path::new("/path/that/definitely/does/not/exist.lua"));

        assert!(result.is_err());
    }
}
