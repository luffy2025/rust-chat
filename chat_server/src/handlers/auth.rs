use crate::{
    error::{AppError, ErrorOutput},
    models::{CreateUser, SigninUser},
    AppState, User,
};
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    if User::find_by_email(&input.email, &state.pool)
        .await?
        .is_some()
    {
        return Err(AppError::EmailAlreadyExists(input.email));
    }

    let user = User::create(&input, &state.pool).await?;
    let token = state.ek.sign(user)?;
    Ok((StatusCode::CREATED, Json(AuthOutput { token })).into_response())
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    match User::verify(&input, &state.pool).await? {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => {
            let body = Json(ErrorOutput::new("invalid email or password"));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppConfig;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = CreateUser::new("Monkey D. Luffy", "luffy@acme.org", "hunter42");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let input = CreateUser::new("Monkey D. Luffy", "luffy@acme.org", "hunter42");
        let _ret = signup_handler(State(state.clone()), Json(input)).await?;

        let input = SigninUser::new("luffy@acme.org", "hunter42");
        let ret = signin_handler(State(state.clone()), Json(input))
            .await?
            .into_response();

        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }
}
