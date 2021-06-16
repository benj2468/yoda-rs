pub type SqlQuery<'a> = sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>;
pub trait Search<S> {
    fn ty() -> String;
    fn array_splits(&self) -> Vec<String>;
    fn paths(&self) -> Vec<String>;
    fn bind_args<'a>(&self, query: SqlQuery<'a>) -> SqlQuery<'a>;
}
