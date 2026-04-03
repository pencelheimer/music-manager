use std::sync::Arc;

use sqlx::SqlitePool;

use crate::LuaVM;

#[derive(Debug)]
pub struct GlobalState {
    pub lua_vm: Arc<LuaVM>,
    pub db_pool: SqlitePool,
}

impl GlobalState {
    pub fn new(lua_vm: LuaVM, db_pool: SqlitePool) -> Self {
        let lua_vm = Arc::new(lua_vm);

        Self { lua_vm, db_pool }
    }
}
