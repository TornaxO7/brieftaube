use ormlite::{Connection, sqlite::SqliteConnection};

pub struct DatabaseSource {
    conn: SqliteConnection,
}

impl DatabaseSource {
    pub async fn new() -> Result<Self, ()> {
        let conn = SqliteConnection::connect(":memory:").await;

        todo!()
    }
}
