mod getter;
mod imp;
use poise::serenity_prelude as serenity;

use std::env;
use tokio;
// use tokio_postgres::{tls::TlsConnect, Client, Connection, Error, NoTls};

// Poise用
// strct.rsへ移動

/*

    独自エラー型の実装が必須
        - postgresのエラー
        - serenityのエラー
        - std::Error

*/

/* 他のモジュールでも使いまわす */
pub struct Data {}
/* エラーハンドル用
    他のモジュールでも使いまわします
*/
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    /* Bot起動フェーズ */
    dotenv::dotenv().ok();
    env_logger::init();
    let token = env::var("TOKEN").expect("missing get token");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![imp::test(), getter::getforum()],
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands)
                    .await
                    .unwrap();
                Ok(Data {})
            })
        });
    framework.run().await.unwrap();
    Ok(())
}

/* 参考
    - 【Rust】 Rust + PostgreSQL + tokio_postgresでDBアクセスする方法
        - https://qiita.com/SakasuRobo/items/a72f916c1e1c8fb63de7
*/
