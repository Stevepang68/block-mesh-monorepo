use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub async fn update_verified_email(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: &Uuid,
    verified_email: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE users SET verified_email = $1 WHERE id = $2"#,
        verified_email,
        user_id
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
