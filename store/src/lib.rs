pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn new() -> Result<Self, std::env::VarError> {
        let url = std::env::var("DATABASE_URL")?;

        Ok(Self { url })
    }
}

pub mod sql;
