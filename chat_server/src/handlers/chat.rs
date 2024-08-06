use crate::{
    error::AppError,
    models::{Chat, CreateChat, UpdateChat, Workspace},
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
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    if chat.ws_id != user.ws_id {
        return Err(AppError::UpdateChatError(
            "Can not update the chat which in other workspace.".to_string(),
        ));
    }

    let chat = Chat::update(id, input, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn delete_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    match Workspace::find_by_id(chat.ws_id, &state.pool).await? {
        Some(ws) => {
            if ws.owner_id != user.id {
                return Err(AppError::DeleteChatError(
                    "Only workspace owner can delete the chat.".to_string(),
                ));
            }
        }
        _ => {
            return Err(AppError::DeleteChatError(
                "Workspace of chat not exist.".to_string(),
            ));
        }
    }

    Chat::delete(id, &state.pool).await?;
    Ok((StatusCode::OK, Json("success".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::ChatType, AppConfig};
    use anyhow::{Ok, Result};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn chat_get_chat_handler_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let id: i64 = 1;
        let ret = get_chat_handler(State(state), Path(id))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret = serde_json::from_slice::<Chat>(&body)?;
        assert_eq!(ret.id, id);
        assert_eq!(ret.members.len(), 5);

        Ok(())
    }

    #[tokio::test]
    async fn chat_update_chat_handler_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let user = User::find_by_email("user1@acme.org", &state.pool)
            .await?
            .unwrap();
        let input = UpdateChat::new("pub", &[1, 2, 3], true);
        let ret = update_chat_handler(Extension(user), State(state), Path(1), Json(input))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes();
        let chat = serde_json::from_slice::<Chat>(&body)?;
        assert_eq!(chat.name.unwrap(), "pub");
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn chat_update_chat_handler_should_not_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let user = User::new(10, "test_user", "test_user@acme.org");
        let input = UpdateChat::new("pub", &[1, 2, 3], true);
        let ret = update_chat_handler(Extension(user), State(state), Path(1), Json(input)).await;
        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn chat_delete_chat_handler_should_not_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let user = User::new(10, "test_user", "test_user@acme.org");
        let ret = delete_chat_handler(Extension(user), State(state), Path(1)).await;
        assert!(ret.is_err());

        Ok(())
    }
}
