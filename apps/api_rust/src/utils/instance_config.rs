use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    entities::instance_configurations, error::AppError, utils::soft_delete::SoftDeleteExt, AppState,
};

pub async fn get_config_value(
    state: &AppState,
    key: &str,
    fallback: Option<&str>,
) -> Result<Option<String>, AppError> {
    let value = instance_configurations::Entity::find()
        .active()
        .filter(instance_configurations::Column::Key.eq(key))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .and_then(|config| config.value)
        .or_else(|| fallback.map(str::to_owned));

    Ok(value)
}
