use crate::{
    error::AppError,
    models::{Chat, CreateChat},
    AppState, User,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

pub(crate) async fn list_chats_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = Chat::fetch_all(user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(input, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn update_chat_handler() -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json("")))
}

pub(crate) async fn delete_chat_handler() -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json("")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppConfig;
    use anyhow::{Ok, Result};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn chat_get_chat_handler_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let id: u64 = 1;
        let ret = get_chat_handler(State(state), Path(id))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret = serde_json::from_slice::<Chat>(&body)?;
        assert_eq!(ret.id as u64, id);
        assert_eq!(ret.members.len(), 5);

        Ok(())
    }
}
