use sqlx::MySqlPool;
use context_async::{Context, Timer};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub user_id: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sql = "SELECT 1 as user_id";

    // create user aaa@'%' identified by 'apsdjfopasjdopfjas!!!!';
    // create database if not exists dev;
    // grant all privileges on dev.* to aaa@'%';
    let pool = MySqlPool::connect("mysql://aaa:apsdjfopasjdopfjas!!!!@localhost:3306/dev").await?;

    let fut = sqlx::query_as(sql).fetch_optional(&pool);

    let timer = Timer::todo();
    let user: Option<User> = timer.handle(fut).await??;
    let user = user.unwrap_or(User { user_id: 111 });

    println!("{}", user.user_id);

    Ok(())
}
