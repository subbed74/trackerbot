use crate::{Context, Error};
use crate::data::{grab_api_data, resolve_ip, Player, Server, Team, TEAMMODES};
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
    todo!()
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
    todo!()
}

/// List all bookmarks for the guild
#[poise::command(
    slash_command,
    check = "info_role"
)]
pub async fn bklist(ctx: Context<'_>) -> Result<(), Error> {
    todo!()
}