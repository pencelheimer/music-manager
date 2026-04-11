use std::process::ExitStatus;

use core_lib::error::LuaError;
use error_set::error_set;

error_set! {
    PluginError := AcoustIdError || FpCalcError || MusicBrainzError || TaggerError || ConfigError || DbError || {
        #[display("Coordinator refused the message: {msg}")]
        CoordinatorDead { msg: String }
    }

    DbError := {
        #[display("Database operation failed: {0}")]
        Sqlx(sqlx::Error),

        #[display("Database migration failed: {0}")]
        Migration(sqlx::migrate::MigrateError),

        #[display("IO error: {0}")]
        Io(std::io::Error)
    }

    ConfigError := {
        #[display("Error retrieving lua config: {0}")]
        Lua(LuaError),

        #[display("Configuration error: {msg}")]
        Other { msg: String }
    }

    TaggerError := {
        #[display("Failed to read/write audio tags: {0}")]
        LoftyError(lofty::error::LoftyError),

        #[display("Failed to download cover art: {0}")]
        NetworkError(reqwest::Error),

        #[display("No primary tag could be created for this file format")]
        UnsupportedFormat,
    }

    MusicBrainzError := {
        #[display("HTTP request failed: {0}")]
        RequestFailed(reqwest::Error),

        #[display("Release not found for MBID: {mbid}")]
        NotFound { mbid: String },
    }

    AcoustIdError := {
        #[display("HTTP request failed: {0}")]
        RequestFailed(reqwest::Error),

        #[display("AcoustID API returned an error status: {msg}")]
        ApiError { msg: String },
    }

    FpCalcError := {
        #[display("Failed to execute fpcalc. Is it installed? (IO Error: {0})")]
        Io(std::io::Error),

        #[display("fpcalc failed with exit status: {status}")]
        ProcessFailed { status: ExitStatus },

        #[display("Failed to parse fpcalc output")]
        ParseError(serde_json::Error),
    }
}

impl FpCalcError {
    pub fn process_failed(status: ExitStatus) -> Self {
        Self::ProcessFailed { status }
    }
}

impl AcoustIdError {
    pub fn api_error(msg: impl ToString) -> Self {
        let msg = msg.to_string();

        Self::ApiError { msg }
    }
}

impl MusicBrainzError {
    pub fn not_found(mbid: impl ToString) -> Self {
        let mbid = mbid.to_string();

        Self::NotFound { mbid }
    }
}

impl PluginError {
    pub fn coordinator_dead(msg: impl ToString) -> Self {
        let msg = msg.to_string();

        Self::CoordinatorDead { msg }
    }
}
