error_set::error_set! {
    LibError := LuaError || DbError

    LuaError := {
        #[display("Lua VM operation failed: {0}")]
        Mlua(mlua::Error),
    }

    DbError := {
        #[display("Database operation failed: {0}")]
        Sqlx(sqlx::Error),

        #[display("Database migration failed: {0}")]
        Migration(sqlx::migrate::MigrateError),

        #[display("IO error: {0}")]
        Io(std::io::Error)
    }
}
