mod disp;
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
    めんどくさいことになりそうだったので, tokio_postgresのエラーをserenityのエラーに置き換えて処理

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

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                imp::test(),
                disp::showall(),
                disp::showtask(),
                imp::addtask(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
    Ok(())
}

/* 参考
    - 【Rust】 Rust + PostgreSQL + tokio_postgresでDBアクセスする方法
        - https://qiita.com/SakasuRobo/items/a72f916c1e1c8fb63de7
*/
