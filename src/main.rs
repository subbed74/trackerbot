use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use tokio::time::Duration;

mod player;
mod server;

pub struct Data {
    // User data, which is stored and accessible in all command invocations
    //database: sqlx::SqlitePool,
    client: reqwest::Client,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn listener(
    _ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        // On bot start
        poise::Event::Ready { .. } => {
            println!("SauerTracker Bot connected!");
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Connect to sqlite DB
    /*let database_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            database_url
                .parse::<sqlx::sqlite::SqliteConnectOptions>().unwrap()
                .create_if_missing(true),
        )
        .await.unwrap();*/
    //sqlx::migrate!("./migrations").run(&database).await.unwrap();

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko)")
        .timeout(Duration::from_secs(600))
        .build()
        .unwrap();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                server::server(),
                server::listservers(),
                player::findplayer(),
                player::player(),
            ],
            event_handler: |ctx, event, _, data| Box::pin(listener(ctx, event, data)),
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MESSAGES,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    //database: database,
                    client,
                })
            })
        });

    framework.run().await.unwrap();
}
