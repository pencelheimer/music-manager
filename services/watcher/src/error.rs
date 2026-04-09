use core_lib::error::LuaError;
use error_set::error_set;

error_set! {
    WatcherError := {
        #[display("Error retrieving config: {0}")]
        Config(LuaError),

        #[display("Error monitoring FS: {0}")]
        Notify(notify::Error),
    }
}
