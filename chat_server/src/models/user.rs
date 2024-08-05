use super::{ChatUser, Workspace};
use crate::{error::AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::mem;

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub workspace: String,
    pub fullname: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            r#"SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn create(input: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        if User::find_by_email(&input.email, pool).await?.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        let password_hash = hash_password(&input.password)?;

        let ws = match Workspace::find_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&input.workspace, 0, pool).await?,
        };

        let ws_id = ws.id;
        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (ws_id, email, fullname, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, created_at
            "#,
        )
        .bind(ws_id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            ws.update_owner(user.id as _, pool).await?;
        }

        Ok(user)
    }

    pub async fn verify(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, ws_id, email, fullname, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(pool)
        .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                if verify_password(&input.password, &password_hash.unwrap_or_default())? {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl ChatUser {
    pub async fn fetch_by_ids(ids: &[i64], pool: &PgPool) -> Result<Vec<Self>, AppError> {
        if ids.is_empty() {
            return Err(AppError::CreateChatError("users is empty".to_string()));
        }

        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    #[allow(unused)]
    pub async fn fetch_all(ws_id: u64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(is_valid)
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
impl CreateUser {
    pub fn new(ws: &str, fullname: &str, email: &str, password: &str) -> Self {
        Self {
            workspace: ws.to_string(),
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::get_test_pool;
    use anyhow::{Ok, Result};

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "hunter42";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(password, &password_hash)?);

        Ok(())
    }

    #[tokio::test]
    async fn create_user_and_verify_user_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let input = CreateUser::new("acme1", "Monkey D. Luffy", "luffy@acme.org", "hunter42");

        let user = User::create(&input, &pool).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);
        assert!(user.id > 0);

        let user = User::find_by_email(&input.email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.fullname, input.fullname);

        let input = SigninUser::new(&input.email, &input.password);
        let user = User::verify(&input, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);

        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let input = CreateUser::new("acme2", "Monkey D. Luffy", "", "hunter42");
        let user = User::create(&input, &pool).await?;
        assert_eq!(user.email, input.email);
        let ret = User::create(&input, &pool).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, input.email);
            }
            _ => panic!("unexpected result: {:?}", ret),
        }

        Ok(())
    }
}
