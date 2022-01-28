use sqlx::PgPool;

use crate::error::Result;

pub async fn save_exception(pool: &PgPool, exception: &serde_json::Value) -> Result<()> {
    sqlx::query("INSERT INTO public.exception (content) VALUES ($1);")
        .bind(exception)
        .execute(pool)
        .await?;
    Ok(())
}
