use crate::{Context, Error};
use crate::data::{escape_markdown, grab_api_data, paginate};
use crate::admin::info_role;

/// Shows a list of similar player names up to 200 names.
#[poise::command(
    slash_command,
    user_cooldown = 10,
    check = "info_role"
)]
pub async fn findplayer(
    ctx: Context<'_>,
    #[description = "Username to search for"]
    #[max_length = 15] username: String,

    #[description = "Country code for user. Use __ for unknown country."]
    #[max_length = 2] country: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let country = if country.is_none() {
        String::new()
    } else {
        country.unwrap().to_ascii_uppercase()
    };

    // Grab Information
    let api_link = format!("http://sauertracker.net/api/v2/players/find?name={username}&country={country}");
    let page_url = format!("https://sauertracker.net/players/find?name={username}&country={country}");

    let data = match grab_api_data(&ctx.data().client, api_link, &page_url).await {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    let data = data.as_array().unwrap();

    // Format information
    let mut page_contents: Vec<String> = Vec::new();
    let mut page = String::new();
    for (i, player) in data.iter().enumerate() {
        page = format!(
            "{}- **[{}]** {}({})\n",
            page,
            i + 1,
            escape_markdown(player["name"].as_str().unwrap().to_string()),
            player["country"].as_str().unwrap()
        );

        if (i + 1) % 10 == 0 {
            page_contents.push(page);
            page = String::new();
        } else if i == data.len() - 1 {
            page_contents.push(page.clone());
        }
    }
    let page_ref: Vec<&str> = page_contents.iter().map(|x| x.as_str()).collect();

    paginate(
        ctx,
        format!("Names similar to {username}"),
        &page_ref,
        Some(page_url),
    )
    .await?;

    Ok(())
}

/// Lookup information on a specific player.
#[poise::command(
    slash_command,
    user_cooldown = 10,
    check = "info_role"
)]
pub async fn player(
    ctx: Context<'_>,
    #[description = "Username of player"]
    #[max_length = 15] username: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Grab and validate information
    let api_link = format!("https://sauertracker.net/api/player/{username}");
    let page_url = format!("https://sauertracker.net/player/{username}");

    let data = match grab_api_data(&ctx.data().client, api_link, &page_url).await {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    let clan = match data["player"]["clan"].as_str() {
        Some(val) => {
            format!("{} - {}", data["player"]["clanTag"].as_str().unwrap(), val)
        }
        None => String::from("None"),
    };

    let country = data["player"]["countryName"].as_str().unwrap_or("Unknown");

    // Organize information
    let duel_stats = format!(
        "Wins: {}\nLosses: {}\nTies: {}\nTotal: {}",
        data["duelStats"]["wins"].as_i64().unwrap(),
        data["duelStats"]["losses"].as_i64().unwrap(),
        data["duelStats"]["ties"].as_i64().unwrap(),
        data["duelStats"]["total"].as_i64().unwrap()
    );

    let desc = format!(
        "**Country:** {}\n**ELO:** {}\n**Games played:** {}\n**Clan:** {}\n",
        country,
        data["player"]["elo"].as_i64().unwrap(),
        data["totalGames"].as_str().unwrap(),
        clan
    );

    let total_stats = format!(
        "Frags: {}\nDeaths: {}\nTeamkills: {}\nFlags: {}\nK\\D: {}\nAcc: {}%",
        data["player"]["frags"].as_i64().unwrap(),
        data["player"]["deaths"].as_i64().unwrap(),
        data["player"]["tks"].as_i64().unwrap(),
        data["player"]["flags"].as_i64().unwrap(),
        data["player"]["kpd"].as_f64().unwrap(),
        data["player"]["acc"].as_f64().unwrap().trunc() as i64
    );

    let insta_stats = format!(
        "Frags: {}\nDeaths: {}\nTeamkills: {}\nFlags: {}\nK\\D: {}\nAcc: {}%",
        data["player"]["instastats"][0].as_i64().unwrap(),
        data["player"]["instastats"][2].as_i64().unwrap(),
        data["player"]["instastats"][3].as_i64().unwrap(),
        data["player"]["instastats"][1].as_i64().unwrap(),
        data["player"]["instastats"][4].as_f64().unwrap(),
        data["player"]["instastats"][5].as_f64().unwrap().trunc() as i64
    );

    let effic_stats = format!(
        "Frags: {}\nDeaths: {}\nTeamkills: {}\nFlags: {}\nK\\D: {}\nAcc: {}%",
        data["player"]["efficstats"][0].as_i64().unwrap(),
        data["player"]["efficstats"][2].as_i64().unwrap(),
        data["player"]["efficstats"][3].as_i64().unwrap(),
        data["player"]["efficstats"][1].as_i64().unwrap(),
        data["player"]["efficstats"][4].as_f64().unwrap(),
        data["player"]["efficstats"][5].as_f64().unwrap().trunc() as i64
    );

    ctx.send(|m| {
        m.embed(|e| {
            e.colour(0xFF0000);
            e.title(format!("{} stats", escape_markdown(username.clone())));
            e.url(page_url);
            e.description(desc);

            e.field("Duels:", duel_stats, false);

            e.field("Total:", total_stats, true);
            e.field("Insta:", insta_stats, true);
            e.field("Effic:", effic_stats, true)
        })
    })
    .await?;

    Ok(())
}
