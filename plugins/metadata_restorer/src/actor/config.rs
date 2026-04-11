use core_lib::{GlobalState, LuaVM};

use crate::{
    error::{ConfigError, PluginError},
    misc::UserAgent,
};

/// Structure to hold configuration extracted from Lua
pub struct Config {
    /// AcoustID API base url
    pub acoustid_api_url: String,

    /// AcoustID API client key
    pub acoustid_api_key: String,

    /// MusicBrainz API base url
    pub musicbrainz_api_url: String,

    /// Tracks similarity threshold
    pub similarity_threshold: f64,

    /// User Agent to use for making API calls
    pub user_agent: UserAgent,
}

impl Config {
    /// Prepares configuration by setting defaults and extracting values from Lua VM
    pub fn new(state: &GlobalState) -> Result<Config, PluginError> {
        let lua_vm = &state.lua_vm;

        lua_vm.set_default_config_value(
            "metadata_restorer_acoustid_api_url",
            "https://api.acoustid.org/v2/lookup",
        )?;
        lua_vm.set_default_config_value(
            "metadata_restorer_musicbrainz_api_url",
            "https://musicbrainz.org/ws/2",
        )?;
        lua_vm.set_default_config_value("metadata_restorer_similarity_threshold", 0.8)?;
        // TODO(pencelheimer): set default user agent

        Ok(Config {
            acoustid_api_url: lua_vm.acoustid_api_url()?,
            acoustid_api_key: lua_vm.acoustid_api_key()?,
            musicbrainz_api_url: lua_vm.musicbrainz_api_url()?,
            similarity_threshold: lua_vm.similarity_threshold()?,
            user_agent: lua_vm.user_agent()?,
        })
    }
}

/// Extension Trait to extract config for the current plugin from the LuaVM
pub trait LuaMetadataRestorerExt {
    /// Extract the AcoustID API base url
    fn acoustid_api_url(&self) -> Result<String, PluginError>;

    /// Extract the AcoustID API client key
    fn acoustid_api_key(&self) -> Result<String, PluginError>;

    /// Extract the MusicBrainz API base url
    fn musicbrainz_api_url(&self) -> Result<String, PluginError>;

    /// Extract the tracks similarity threshold
    fn similarity_threshold(&self) -> Result<f64, PluginError>;

    /// Extract the user agent
    fn user_agent(&self) -> Result<UserAgent, PluginError>;
}

impl LuaMetadataRestorerExt for LuaVM {
    fn acoustid_api_url(&self) -> Result<String, PluginError> {
        let url = self.get_config_value("metadata_restorer_acoustid_api_url")?;
        Ok(url)
    }

    fn acoustid_api_key(&self) -> Result<String, PluginError> {
        let key = self.get_config_value("metadata_restorer_acoustid_api_key")?;
        Ok(key)
    }

    fn musicbrainz_api_url(&self) -> Result<String, PluginError> {
        let url = self.get_config_value("metadata_restorer_musicbrainz_api_url")?;
        Ok(url)
    }

    fn similarity_threshold(&self) -> Result<f64, PluginError> {
        let t = self.get_config_value("metadata_restorer_similarity_threshold")?;
        Ok(t)
    }

    fn user_agent(&self) -> Result<UserAgent, PluginError> {
        let agent_str: String = self.get_config_value("metadata_restorer_user_agent")?;
        let agent = UserAgent::try_new(agent_str).map_err(|e| ConfigError::Other {
            msg: format!("Invalid UserAgent: {e}"),
        })?;
        Ok(agent)
    }
}
