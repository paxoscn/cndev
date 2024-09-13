
use cndev_service::sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub templates: tera::Tera,
    pub conn: DatabaseConnection,
    pub redis: redis::Client,
}
