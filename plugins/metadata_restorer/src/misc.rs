pub use user_agent::UserAgent;

mod user_agent {
    use nutype::nutype;

    #[nutype(
        validate(regex = r#"^[^/]+/[^\s]+ \(\s*[^@]+@[^@]+\.[^@]+\s*\)$"#),
        derive(AsRef, Deref, Display, Debug, Clone)
    )]
    pub struct UserAgent(String);
}
