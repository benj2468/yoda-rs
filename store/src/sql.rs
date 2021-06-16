use std::str::FromStr;

use atoms::{
    delta::{Del, Store},
    pagination::PaginationOption,
    search::Search,
};
use serde::Serialize;
use sqlx::{types::Uuid, Row};

fn convert_id(id: &str) -> sqlx::Result<Uuid> {
    sqlx::types::Uuid::from_str(id).map_err(|e| sqlx::Error::Configuration(Box::new(e)))
}

pub struct Driver;

impl Driver {
    pub async fn query_proj<T, S>(
        pool: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &str,
    ) -> sqlx::Result<T>
    where
        S: serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug + Store,
        T: Del<S>,
    {
        let sqlx::types::Json(res): sqlx::types::Json<S> = sqlx::query::<sqlx::Postgres>(
            "
            SELECT body FROM projection p
            WHERE ty = $1
            AND id = $2
        ",
        )
        .bind(S::ty())
        .bind(convert_id(id)?)
        .fetch_one(pool)
        .await?
        .try_get("body")?;

        Ok(res.into())
    }
    pub async fn query<T, S>(pool: &sqlx::PgPool, id: &str) -> sqlx::Result<Vec<S>>
    where
        S: serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug,
        T: Del<S>,
    {
        sqlx::query::<sqlx::Postgres>(
            "
            SELECT body FROM delta d
            WHERE ty = $1
            AND id = $2
            order by created_at
        ",
        )
        .bind(T::ty())
        .bind(convert_id(id)?)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| -> sqlx::Result<S> {
            let sqlx::types::Json(res): sqlx::types::Json<S> = row.try_get("body")?;
            Ok(res)
        })
        .collect()
    }

    pub async fn search<T, S>(
        pool: &sqlx::PgPool,
        doc: T,
        pagination: PaginationOption,
    ) -> sqlx::Result<Vec<S>>
    where
        T: Search<S>,
        S: serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug,
    {
        let array_breakups = doc.array_splits().join("");

        let mut paths = doc
            .paths()
            .into_iter()
            .enumerate()
            .map(|(i, path)| path.replace('$', format!("${}", i + 2).as_str()))
            .collect::<Vec<_>>()
            .join("AND ");

        if !paths.is_empty() {
            paths = format!("AND {}", paths)
        }

        let query_str = format!(
            "
                SELECT body FROM projection p
                {}
                WHERE ty = $1
                {}
                ORDER BY last_updated
                LIMIT {}
                OFFSET {}
            ",
            array_breakups, paths, pagination.limit, pagination.skip
        );

        let query = sqlx::query(query_str.as_str()).bind(T::ty());

        doc.bind_args(query)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|row| {
                let sqlx::types::Json(res): sqlx::types::Json<S> = row.try_get("body")?;
                Ok(res)
            })
            .collect()
    }

    pub async fn delta<S>(
        pool: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &str,
        doc: S,
        author: &auth::Identity,
    ) -> sqlx::Result<()>
    where
        S: Serialize + Send + Store,
    {
        sqlx::query(
            "
            INSERT INTO delta
            (id, ty, body, author)
            VALUES ($1, $2, $3, $4)
        ",
        )
        .bind(convert_id(id)?)
        .bind(S::ty())
        .bind(sqlx::types::Json(doc))
        .bind(&author.user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn project<S>(
        pool: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &str,
        doc: S,
    ) -> sqlx::Result<()>
    where
        S: Serialize + Send + Store,
    {
        sqlx::query(
            "
            INSERT INTO projection
            (id, ty, body)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET body = EXCLUDED.body
        ",
        )
        .bind(convert_id(id)?)
        .bind(S::ty())
        .bind(sqlx::types::Json(doc))
        .execute(pool)
        .await?;

        Ok(())
    }
}
