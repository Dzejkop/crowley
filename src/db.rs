use sqlx::{Connection, SqliteConnection};

const DATABASE_FILE: &str = "./database.db";

pub async fn get_connection() -> anyhow::Result<SqliteConnection> {
    Ok(SqliteConnection::connect(DATABASE_FILE).await?)
}
