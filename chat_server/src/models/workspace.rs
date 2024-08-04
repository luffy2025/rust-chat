use super::{ChatUser, Workspace};
use crate::error::AppError;
use sqlx::PgPool;

impl Workspace {
    pub async fn create(name: &str, user_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspaces SET owner_id=$1
            WHERE id = $2 and (SELECT ws_id FROM users where id = $1) = $2
            RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(unused)]
    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(unused)]
    pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE
            ws_id = $1
            ORDER BY id
        "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::CreateUser, test_util::get_test_pool, User};
    use anyhow::{Ok, Result};

    #[tokio::test]
    async fn workspace_create_and_set_owner_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let ws = Workspace::create("test", 0, &pool).await?;
        assert_eq!(ws.name, "test");

        let input = CreateUser::new(&ws.name, "test user", "testUser@acme.org", "hunter42");
        let user = User::create(&input, &pool).await?;
        assert_eq!(user.ws_id, ws.id);

        let ws = ws.update_owner(user.id as _, &pool).await?;
        assert_eq!(ws.owner_id, user.id);
        assert_eq!(user.ws_id, ws.id);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_id_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let ws = Workspace::find_by_id(1, &pool).await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "workspace1");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_name_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let ws = Workspace::find_by_name("workspace2", &pool).await?;
        assert!(ws.is_some());
        assert_eq!(ws.unwrap().name, "workspace2");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_fetch_all_chat_users_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let users = Workspace::fetch_all_chat_users(1, &pool).await?;
        assert_eq!(users.len(), 5);
        assert_eq!(users[2].fullname, "user3");
        Ok(())
    }
}
