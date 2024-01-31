use crate::{Context, Error};
use crate::data::grab_api_data;
use crate::admin::info_role;
use poise::serenity_prelude as serenity;

/// Display basic information about a clan
#[poise::command(
    slash_command,
    user_cooldown = 10,
    check = "info_role"
)]
pub async fn claninfo(
    ctx: Context<'_>,
    #[description = "Clantag to search."] clantag: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Grab and validate information
    let api_link = format!("https://sauertracker.net/api/clan/{clantag}");
    let page_url = format!("https://sauertracker.net/clan/{clantag}");

    let data = match grab_api_data(&ctx.data().client, api_link, &page_url).await
    {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    if data["error"].as_str().is_some() {
        if data["error"].as_str().unwrap() == "Clan not found." {
            return Err("No clan found with that clantag!".into());
        }
    }

    // Organize display information
    let title = format!("{} - {}",
        data["info"]["tag"].as_str().unwrap(),
        data["info"]["title"].as_str().unwrap());

    let mut lastseen = String::new();
    for (i, member) in data["members"].as_array().unwrap().iter().enumerate() {
        let last = if i == data["members"].as_array().unwrap().len() - 1 {
            String::new()
        } else {
            String::from(", ")
        };

        lastseen = format!("{lastseen}{}{}",
            member["name"].as_str().unwrap(),
            last);
    }

    // Grab clanwar info
    let games = data["games"].as_array().unwrap();
    let most_recent_cw = if games.len() == 0 {
        String::new()
    } else {
        let mut list_str = String::new();

        for (_, game) in games.iter().enumerate() {
            // Swap meta values for winner on the left
            let mut meta: [String; 4] = [game["meta"][0].as_str().unwrap().to_string(), game["meta"][1].as_i64().unwrap().to_string(), game["meta"][2].as_str().unwrap().to_string(), game["meta"][3].as_i64().unwrap().to_string()];
            if meta[3].parse::<i64>().unwrap() > meta[1].parse::<i64>().unwrap() {
                meta.swap(0, 2);
                meta.swap(1, 3);
            }

            list_str = format!("{}- **{}** ({}) v. **{}** ({}) - *{} {}* [More info...](https://sauertracker.net/game/{})\n",
                list_str,
                meta[0],
                meta[1],
                meta[2],
                meta[3],
                game["gamemode"].as_str().unwrap(),
                game["map"].as_str().unwrap(),
                game["id"].as_i64().unwrap()
            );
        }

        format!("__**Most recent clanwars:**__\n{list_str}")
    };

    let desc = format!("**Website:** {}\n**Wins:** {}\n**Losses:** {}\n**Ties:** {}\n\n**Recently seen:** {}\n\n{}",
        data["info"]["website"].as_str().unwrap_or("None"),
        data["clan"]["wins"].as_i64().unwrap(),
        data["clan"]["losses"].as_i64().unwrap(),
        data["clan"]["ties"].as_i64().unwrap(),
        lastseen,
        most_recent_cw
    );

    // Build embed
    let clan_embed = serenity::CreateEmbed::new()
        .title(title)
        .url(page_url)
        .description(desc);

    // Display information
    ctx.send(poise::CreateReply::default().embed(clan_embed)).await?;

    Ok(())
}