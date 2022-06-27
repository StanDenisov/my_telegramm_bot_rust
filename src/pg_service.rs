use sqlx::postgres::PgPool;

#[derive(Debug, Clone)]
pub struct PgService {
    pub pg_pool: PgPool,
}

impl PgService {
    pub async fn new() -> Self {
        let pg_pool = PgPool::connect(&*dotenv::var("DATABASE_URL").unwrap())
            .await
            .expect("Cant connect to the pg server");
        Self { pg_pool }
    }
}
