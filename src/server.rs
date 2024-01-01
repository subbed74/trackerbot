//use poise::serenity_prelude as serenity;
use crate::{Context, Error};
use crate::data::{grab_api_data, resolve_ip, ServerPlayer, DetailedServer, BasicServer, TEAMMODES};
use crate::admin::info_role;
use serde_json::Value;

/// Grab active servers.
#[poise::command(
    slash_command,
    user_cooldown = 10,
    check = "info_role",
    guild_only
)]
pub async fn listservers(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let api_link = String::from("https://sauertracker.net/api/v2/servers");
    let page_url = String::from("https://sauertracker.net");

    let server_data = match grab_api_data(&ctx.data().client, api_link, &page_url).await {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    let mut server_vec: Vec<BasicServer> = Vec::new();
    for server in server_data.as_array().unwrap() {
        if server["clients"].as_i64().unwrap() == 0 || server["version"].as_i64().unwrap() < 260 {
            continue;
        }

        server_vec.push(serde_json::from_value(server.clone()).unwrap());
    }

    server_vec.sort_by(|a, b| b.clients.cmp(&a.clients));
    server_vec.truncate(10);

    // Format data into list
    let mut server_list: String = String::from("__**Active Servers:**__\n");
    for (i, server) in server_vec.iter().enumerate() {
        let inc_port = if server.port != 28785 {
            format!(" port:{}", server.port)
        } else {
            String::new()
        };

        server_list = format!("{}- **[{}]** [{}](https://sauertracker.net/server/{}/{}) - Info: `/server host:{}{}`\n - {}/{} | {} {} - {} | {}\n",
            server_list,
            i+1,
            server.description,
            server.host,
            server.port,
            server.host,
            inc_port,
            server.clients,
            server.maxClients,
            server.gameMode,
            server.mapName,
            server.timeLeftString,
            server.masterMode,
        );
    }

    //println!("{:#?}", server_vec);

    ctx.say(server_list).await?;
    Ok(())
}

/// Grab information on a server.
#[poise::command(
    slash_command,
    user_cooldown = 10,
    check = "info_role",
    guild_only
)]
pub async fn server(
    ctx: Context<'_>,
    #[description = "Server Addr"] host: String,
    #[description = "Server Port"] port: Option<u32>,
    #[description = "Player in game"]
    #[max_length = 15] username: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let port = port.unwrap_or(28785_u32);
    let page_url = format!("https://sauertracker.net/server/{host}/{port}");

    let server_data = match get_server_info(&ctx.data().client, host.clone(), port.clone()).await {
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

    // If username not specified, show full server otherwise user stats
    if username.is_none() {
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

                e.footer(|f| f.text(format!("/connect {host} {port}")))
            })
        })
        .await?;
    } else {
        // Check if player is in server
        let mut player_stats: Option<ServerPlayer> = None;
        for player in server_data.players {
            if player.name == username.clone().unwrap() {
                player_stats = Some(player);
                break;
            }
        }

        if player_stats.is_none() {
            return Err(format!("{}, that player was not found in the server!", ctx.author()).into());
        }

        // Build and send embed if found
        embed_desc = format!("{embed_desc}__**Player:**__\n{}",
            player_stats.as_ref().unwrap().name
        );



        ctx.send(|m| {
            m.embed(|e| {
                e.colour(0xFF0000);
                e.title(server_data.description);
                e.url(page_url);
                e.description(embed_desc);

                e.field("Frags:", player_stats.as_ref().unwrap().frags, true);
                e.field("Deaths:", player_stats.as_ref().unwrap().deaths, true);
                e.field("KpD:", player_stats.as_ref().unwrap().kpd, true);

                e.field("Accuracy:", player_stats.as_ref().unwrap().acc, true);
                e.field("Flags:", player_stats.as_ref().unwrap().flags, true);
                e.footer(|f| f.text(format!("/connect {host} {port}")))
            })
        })
        .await?;
    }

    Ok(())
}

// Get server info container
pub async fn get_server_info(client: &reqwest::Client, host: String, port: u32) -> Result<DetailedServer, Error> {
    // Validate host
    let host = match resolve_ip(host.clone()).await {
        Some(ip) => ip,
        None => return Err("Unable to resolve server address!".into()),
    };

    // Check if the server exists
    let api_link = String::from("https://sauertracker.net/api/v2/servers");
    let page_url = String::from("https://sauertracker.net");

    let all_server_data = match grab_api_data(client, api_link, &page_url).await {
        Ok(data) => data,
        Err(_) => return Err("There was an error pulling information for servers!".into()),
    };

    let all_server_data = all_server_data.as_array().unwrap();
    if !server_exists(all_server_data, &host, port) {
        return Err("The server you have specified does not exist!".into());
    }

    // Grab and parse data
    let api_url = format!("http://sauertracker.net/api/v2/server/{host}/{port}");
    let page_url = format!("https://sauertracker.net/server/{host}/{port}");

    let server_data = match grab_api_data(client, api_url, &page_url).await {
        Ok(data) => data,
        Err(_) => return Err("There was an error pullling server information!".into())
    };

    let mut server_data: DetailedServer = serde_json::from_value(server_data).unwrap();

    // Populate spectator/team player vectors
    if TEAMMODES.contains(&server_data.gameMode.as_str()) {
        for team in &mut server_data.teams {
            let mut team_players: Vec<String> = Vec::new();

            for player in &server_data.players {
                if player.team == team.name && player.state != 5 {
                    team_players.push(player.name.clone());
                }
            }

            team.players = Some(team_players);
        }
    }

    let mut spec_vec: Vec<String> = Vec::new();
    let mut active_players: Vec<String> = Vec::new();
    for player in &server_data.players {
        if player.state == 5 {
            spec_vec.push(player.name.clone());
        } else {
            active_players.push(player.name.clone());
        }
    }
    server_data.spectators = Some(spec_vec);
    server_data.all_active_players = Some(active_players);

    Ok(server_data)
}

pub fn server_exists(server_array: &Vec<Value>, host: &String, port: u32) -> bool {
    for server in server_array {
        if server["host"].as_str().unwrap() == host.as_str() && server["port"].as_i64().unwrap() as u32 == port {
            return true;
        }
    }

    false
}