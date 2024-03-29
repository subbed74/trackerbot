use crate::{Context, Error};
use poise::serenity_prelude as serenity;

// -- Handling role requirements for information commands.
/// Set a required role to run info commands.
#[poise::command(
    slash_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn setrole(
    ctx: Context<'_>,
    #[description = "Chosen role required to run info commands. Leave empty for no role."] req_role: Option<serenity::Role>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    let role_id: Option<u64> = match &req_role {
        Some(role) => Some(role.id.get()),
        None => None
    };

    sqlx::query!("UPDATE guild_settings SET infocmds_required_role = ? WHERE guild_id = ?", role_id, guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    // Check if empty, set role to null if so
    if req_role.is_none() {
        ctx.say(format!("{}. information commands do not require a role!", ctx.author())).await?;
    } else {
        ctx.say(format!("{}, information commands now require the {} role!", ctx.author(), req_role.unwrap().name)).await?;
    }

    Ok(())
}

// Command check for required roles
pub async fn info_role(ctx: Context<'_>) -> Result<bool, Error> {
    // Pull data and validate
    if ctx.partial_guild().await.is_none() {
        return Ok(true);
    }

    let guild_id = ctx.guild_id().unwrap().get();
    let role_id = sqlx::query!("SELECT infocmds_required_role AS id FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if role_id.id.is_none() { 
        return Ok(true);
    }

    // Check user roles
    let author = ctx.author_member().await.unwrap();
    if author.roles.contains(&serenity::RoleId::new(role_id.id.unwrap())) {
        Ok(true)
    } else {
        ctx.send(poise::CreateReply::default()
            .content(format!("{}, you do not have permission to run this command!", ctx.author()))
            .ephemeral(true)
        ).await?;

        Ok(false)
    }
}