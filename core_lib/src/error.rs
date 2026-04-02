error_set::error_set! {
    LibError := LuaError

    LuaError := {
        #[display("Lua VM operation failed: {0}")]
        Mlua(mlua::Error),
    }
}
