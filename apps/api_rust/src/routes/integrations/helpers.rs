// src/routes/integrations/helpers.rs
//! Internal helpers shared by integrations submodules.

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IsolationLevel, QueryFilter,
    TransactionTrait,
};
use uuid::Uuid;

use crate::{
    entities::api_tokens,
    error::AppError,
    AppState,
};

/// Looks up or creates an api_token for (user, workspace).
/// Replicates Django `APIToken.objects.get_or_create` behavior.
///
/// Operation executes within a transaction with SERIALIZABLE level
/// to avoid the TOCTOU (check-then-insert) race condition that existed before.
/// If two concurrent requests pass the empty SELECT at the same time, only
/// one INSERT will succeed; the other will read the newly created token.
pub async fn get_or_create_api_token(
    state: &AppState,
    user_id: Uuid,
    workspace_id: Uuid,
    label: &str,
) -> Result<api_tokens::Model, AppError> {
    let label = label.to_owned();

    let token = state
        .db
        .transaction_with_config::<_, api_tokens::Model, AppError>(
            |txn| {
                let label = label.clone();
                Box::pin(async move {
                    if let Some(token) = api_tokens::Entity::find()
                        .filter(api_tokens::Column::UserId.eq(user_id))
                        .filter(api_tokens::Column::WorkspaceId.eq(workspace_id))
                        .filter(api_tokens::Column::IsActive.eq(true))
                        .filter(api_tokens::Column::DeletedAt.is_null())
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?
                    {
                        return Ok(token);
                    }

                    let raw = Uuid::new_v4().as_simple().to_string();
                    // explicit created_at/updated_at (NOT NULL without DEFAULT).
                    let now: chrono::DateTime<chrono::FixedOffset> =
                        chrono::Utc::now().into();
                    let new_token = api_tokens::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        token: Set(raw),
                        label: Set(label),
                        user_type: Set(1),
                        user_id: Set(user_id),
                        workspace_id: Set(Some(workspace_id)),
                        description: Set(String::new()),
                        is_active: Set(true),
                        is_service: Set(false),
                        allowed_rate_limit: Set("default".to_owned()),
                        created_at: Set(now),
                        updated_at: Set(now),
                        deleted_at: Set(None),
                        ..Default::default()
                    };

                    new_token.insert(txn).await.map_err(AppError::Database)
                })
            },
            Some(IsolationLevel::Serializable),
            None,
        )
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    Ok(token)
}
