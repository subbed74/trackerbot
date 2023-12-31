use crate::{Context, Error};
use crate::data::ServerBookmark;
use crate::admin::info_role;
use serde_json::Value;

/// Grab server information from a bookmark
#[poise::command(
    slash_command,
    check = "info_role",
    user_cooldown = 10,
    guild_only
)]
pub async fn bk(
    ctx: Context<'_>,
    #[description = "Name of the server bookmark"] bookmark: String
) -> Result<(), Error> {
    todo!()
}

/// Create a server bookmark
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn bkadd(
    ctx: Context<'_>,
    #[description = "Name for the bookmark"] bookmark: String,
    #[description = "Server address/ip"] host: String,
    #[description = "Server port (Default: 28785)"] port: Option<i64>
) -> Result<(), Error> {
    let guild_id = *ctx.guild_id().unwrap().as_u64() as i64;

    let port = match port {
        Some(port) => port,
        None => 28785
    };

    // Verify information in DB
    let count = sqlx::query!("SELECT COUNT(bookmark_name) AS count FROM server_bookmarks WHERE guild_id = ? AND bookmark_name = ?", guild_id, bookmark)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if count.count > 0 {
        return Err("That name is already used!".into());
    }

    let count = sqlx::query!("SELECT COUNT(host) AS count FROM server_bookmarks WHERE guild_id = ? AND host = ? AND port = ?", guild_id, host, port)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if count.count > 0 {
        return Err("That server is already bookmarked!".into());
    }

    // Add to DB
    sqlx::query!("INSERT INTO server_bookmarks (guild_id, bookmark_name, host, port) VALUES (?, ?, ?, ?)", guild_id, bookmark, host, port)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    let msg = format!("{}, created a server bookmark named {}! Address: {}:{}", ctx.author(), bookmark, host, port);
    ctx.say(msg).await?;

    Ok(())
}

/// Remove a server bookmark
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn bkdelete(
    ctx: Context<'_>,
    #[description = "Name of the server bookmark"] bookmark: String
) -> Result<(), Error> {
    let guild_id = *ctx.guild_id().unwrap().as_u64() as i64;

    // Verify in DB
    let count = sqlx::query!("SELECT COUNT(bookmark_name) AS count FROM server_bookmarks WHERE guild_id = ? AND bookmark_name = ?", guild_id, bookmark)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if count.count == 0 {
        return Err("No bookmark with that name exists!".into());
    }

    // Remove entry
    sqlx::query!("DELETE FROM server_bookmarks WHERE guild_id = ? AND bookmark_name = ?", guild_id, bookmark)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    let msg = format!("{}, deleted {} from the server bookmarks!", ctx.author(), bookmark);
    ctx.say(msg).await?;

    Ok(())
}

/// List all bookmarks for the guild
#[poise::command(
    slash_command,
    check = "info_role"
)]
pub async fn bklist(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = *ctx.guild_id().unwrap().as_u64() as i64;

    // Query DB and verify
    let server_bookmarks: Vec<ServerBookmark> = sqlx::query_as!(ServerBookmark, "SELECT * FROM server_bookmarks WHERE guild_id = ?", guild_id)
        .fetch_all(&ctx.data().database)
        .await
        .unwrap();

    if server_bookmarks.is_empty() {
        return Err("No bookmarks saved!".into());
    }

    // Build, send message
    let mut bk_list = String::new();
    for (_, bookmark) in server_bookmarks.iter().enumerate() {
        bk_list = format!("{bk_list}\n- {0} - `/server ip:{1} port:{2}`",
            bookmark.bookmark_name,
            bookmark.host,
            bookmark.port);
    }

    let msg = format!("__**Server Bookmarks:**__{bk_list}");
    ctx.say(msg).await?;

    Ok(())
}