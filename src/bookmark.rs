use crate::{Context, Error};
use crate::data::{TEAMMODES, ServerBookmark};
use crate::admin::info_role;
use crate::server::get_server_info;
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
    let guild_id = *ctx.guild_id().unwrap().as_u64() as i64;

    // Verify then grab information in DB
    let count = sqlx::query!("SELECT COUNT(bookmark_name) AS count FROM server_bookmarks WHERE guild_id = ? AND bookmark_name = ?", guild_id, bookmark)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if count.count == 0 {
        return Err("No bookmark by that name exists!".into());
    }

    let bookmark_info = sqlx::query_as!(ServerBookmark, "SELECT * FROM server_bookmarks WHERE guild_id = ? AND bookmark_name = ?", guild_id, bookmark)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    let page_url = format!("https://sauertracker.net/server/{}/{}", bookmark_info.host, bookmark_info.port);

    let server_data = match get_server_info(&ctx.data().client, bookmark_info.host.clone(), bookmark_info.port.clone()).await {
        Ok(data) => data,
        Err(e) => return Err(e)
    };

    // Format server info
    let mut embed_desc = format!(
        "**Players:** {}/{}\n**Mastermode:** {}\n*{} {} {}*\n\n",
        server_data.clients,
        server_data.maxClients,
        server_data.masterMode,
        server_data.mapName,
        server_data.gameMode,
        if server_data.gameMode != "coop_edit" {
            format!("- {}", server_data.timeLeftString)
        } else {
            String::new()
        }
    );

    ctx.send(|m| {
        m.embed(|e| {
            e.colour(0xFF0000);
            e.title(server_data.description);
            e.url(page_url);
            e.description(embed_desc);

            if TEAMMODES.contains(&server_data.gameMode.as_str()) {
                for team in &server_data.teams {
                    let mut team_players_display = String::new();
                    for player in team.players.clone().unwrap() {
                        team_players_display = format!("{}{}\n", team_players_display, player);
                    }

                    e.field(
                        format!("{}: [{}]", team.name, team.score),
                        team_players_display,
                        true,
                    );
                }
            } else {
                let mut players_display = String::new();
                for player in &server_data.all_active_players.unwrap() {
                    players_display = format!("{}{}\n", players_display, player);
                }

                e.field("Players:", players_display, false);
            }

            if !server_data.spectators.clone().unwrap().is_empty() {
                let mut spec_display = String::new();
                for (i, player) in server_data.spectators.clone().unwrap().iter().enumerate() {
                    if i == server_data.spectators.clone().unwrap().len() - 1 {
                        spec_display = format!("{}{}", spec_display, player);
                    } else {
                        spec_display = format!("{}{}, ", spec_display, player);
                    }
                }

                e.field("Spectators:", spec_display, false);
            }

            e.footer(|f| f.text(format!("/connect {} {}", bookmark_info.host, bookmark_info.port)))
        })
    })
    .await?;

    Ok(())
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
    #[description = "Server port (Default: 28785)"] port: Option<u32>
) -> Result<(), Error> {
    let guild_id = *ctx.guild_id().unwrap().as_u64();

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
    let guild_id = *ctx.guild_id().unwrap().as_u64();

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
    let guild_id = *ctx.guild_id().unwrap().as_u64();

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
        bk_list = format!("{bk_list}\n- {0} - `/server host:{1} port:{2}`",
            bookmark.bookmark_name,
            bookmark.host,
            bookmark.port);
    }

    let msg = format!("__**Server Bookmarks:**__{bk_list}");
    ctx.say(msg).await?;

    Ok(())
}