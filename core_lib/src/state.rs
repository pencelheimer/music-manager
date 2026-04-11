use std::sync::Arc;

use sqlx::SqlitePool;

use crate::LuaVM;

/// The globally shared state of the daemon.
///
/// This struct holds the core resources required by various components and plugins,
/// such as the Lua virtual machine for configuration parsing and the SQLite database
/// pool. It is designed to be cheaply cloned across multiple threads and actors.
#[derive(Debug, Clone)]
pub struct GlobalState {
    /// The thread-safe wrapper around the Lua Virtual Machine.
    pub lua_vm: Arc<LuaVM>, // TODO(pencelheimer): is Arc needed?

    /// The async connection pool to the SQLite database.
    pub db_pool: SqlitePool,
}

impl GlobalState {
    /// Creates a new `GlobalState` instance.
    pub fn new(lua_vm: LuaVM, db_pool: SqlitePool) -> Self {
        let lua_vm = Arc::new(lua_vm);

        Self { lua_vm, db_pool }
    }
}
