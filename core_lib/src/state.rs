use crate::LuaVM;

#[derive(Debug)]
pub struct GlobalState {
    pub lua_vm: LuaVM,
}

impl GlobalState {
    pub fn new(lua_vm: LuaVM) -> Self {
        Self { lua_vm }
    }
}
