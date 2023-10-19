use serde::{Deserialize, Serialize};
use tokio;
use tokio_postgres::{tls::TlsConnect, Client, Connection, Error, NoTls};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, conn) = tokio_postgres::Config::new()
        .user("postgres")
        .password("password")
        .host("localhost")
        .port(5432)
        .dbname("testdb")
        .connect(tokio_postgres::NoTls)
        .await?;

    // 接続タスク実行
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection err: {}", e);
        }
    });
    println!("Connected to psql");

    // レコードを取ってくる
    let rows = client.query("select * from users", &[]).await?;

    // 表示
    for row in rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        println!("id: {}, name: {}", id, name);
    }

    Ok(())
}

/* 参考
    - 【Rust】 Rust + PostgreSQL + tokio_postgresでDBアクセスする方法
        - https://qiita.com/SakasuRobo/items/a72f916c1e1c8fb63de7
*/
