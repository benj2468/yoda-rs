use schema::YodaSchema;

pub struct Config {
    db_config: store::DatabaseConfig,
}

impl Config {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            db_config: store::DatabaseConfig::new()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
        })
    }

    pub async fn connect_db(&self) -> Result<sqlx::PgPool, std::io::Error> {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(self.db_config.url.as_str())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub async fn gen_schema(&self) -> YodaSchema {
        YodaSchema::build(Default::default(), Default::default(), Default::default())
            .data(self.connect_db().await.unwrap())
            .finish()
    }
}
