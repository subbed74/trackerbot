//use poise::serenity_prelude as serenity;
use crate::{Context, Error};
use crate::data::{grab_api_data, resolve_ip, Player, Server, Team, TEAMMODES};
use crate::admin::info_role;
use serde_json::Value;

/// Grab active servers.
#[poise::command(
    slash_command,
    user_cooldown = 10,
    check = "info_role"
)]
pub async fn listservers(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    // Grab data
    /*let server_data = match grab_api_data(
        &ctx.data().client,
        String::from("https://sauertracker.net/api/v2/servers"),
    )
    .await
    {
        Some(data) => data,
        None => return Err("Unable to retrieve data!".into()),
    };*/
    let api_link = String::from("https://sauertracker.net/api/v2/servers");
    let page_url = String::from("https://sauertracker.net");

    let server_data = match grab_api_data(&ctx.data().client, api_link, &page_url).await
    {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    let mut server_vec: Vec<Server> = Vec::new();
    for server in server_data.as_array().unwrap() {
        if server["clients"].as_i64().unwrap() == 0 || server["version"].as_i64().unwrap() < 260 {
            continue;
        }

        server_vec.push(Server {
            description: server["description"].as_str().unwrap().to_string(),
            host: server["host"].as_str().unwrap().to_string(),
            port: server["port"].as_i64().unwrap(),
            clients: server["clients"].as_i64().unwrap(),
            max_clients: server["maxClients"].as_i64().unwrap(),
            gamemode: server["gameMode"].as_str().unwrap().to_string(),
            mapname: server["mapName"].as_str().unwrap().to_string(),
            mastermode: server["masterMode"].as_str().unwrap().to_string(),
            time_left_string: server["timeLeftString"].as_str().unwrap().to_string(),
        });
    }
    server_vec.truncate(10);
    server_vec.sort_unstable_by_key(|server| server.clients);
    server_vec.reverse();

    // Format data into list
    let mut server_list: String = String::from("__**Active Servers:**__\n");
    for (i, server) in server_vec.iter().enumerate() {
        let inc_port = if server.port != 28785 {
            format!(" port: {}", server.port)
        } else {
            String::new()
        };

        server_list = format!("{}- **[{}]** [{}](https://sauertracker.net/server/{}/{}) - Info: `/server ip:{}{}`\n - {}/{} | {} {} - {} | {}\n",
            server_list,
            i+1,
            server.description,
            server.host,
            server.port,
            server.host,
            inc_port,
            server.clients,
            server.max_clients,
            server.gamemode,
            server.mapname,
            server.time_left_string,
            server.mastermode,
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
    check = "info_role"
)]
pub async fn server(
    ctx: Context<'_>,
    #[description = "Server Addr"] ip: String,
    #[description = "Server Port"] port: Option<i64>,
    #[description = "Player in game"]
    #[max_length = 15] username: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Validate args
    let ip = match resolve_ip(ip).await {
        Some(ip) => ip,
        None => return Err("Unable to resolve address!".into()),
    };

    let port = port.unwrap_or(28785_i64);

    // Check if the server exists
    let api_link = String::from("https://sauertracker.net/api/v2/servers");
    let page_url = String::from("https://sauertracker.net");

    let all_server_data = match grab_api_data(&ctx.data().client, api_link, &page_url).await {
        Ok(data) => data,
        Err(_) => return Err("There was an error pulling information for servers!".into()),
    };

    let all_server_data = all_server_data.as_array().unwrap();
    if !server_exists(all_server_data, &ip, port) {
        return Err("The server you have specified does not exist!".into());
    }

    // Begin data handling
    let api_url = format!("http://sauertracker.net/api/v2/server/{ip}/{port}");
    let page_url = format!("https://sauertracker.net/server/{ip}/{port}");

    let server_data = match grab_api_data(&ctx.data().client, api_url, &page_url).await {
        Ok(data) => data,
        Err(err) => return Err(err),
    };

    // Format server info
    let mut embed_desc = format!(
        "**Players:** {}/{}\n**Mastermode:** {}\n*{} {} {}*\n\n",
        server_data["clients"].as_i64().unwrap(),
        server_data["maxClients"].as_i64().unwrap(),
        server_data["masterMode"].as_str().unwrap(),
        server_data["mapName"].as_str().unwrap(),
        server_data["gameMode"].as_str().unwrap(),
        if server_data["gameMode"].as_str().unwrap() != "coop_edit" {
            format!("- {}", server_data["timeLeftString"].as_str().unwrap())
        } else {
            String::new()
        }
    );

    let mut player_vec: Vec<Player> = Vec::new();
    for player in server_data["players"].as_array().unwrap() {
        player_vec.push(serde_json::from_value(player.clone()).unwrap());
    }

    // If username not specified, show full server otherwise user stats
    if username.is_none() {
        // Format teams and a display
        let mut team_vec: Vec<Team> = Vec::new();
        let mut spec_vec: Vec<String> = Vec::new();
        let mut active_vec: Vec<String> = Vec::new();

        if TEAMMODES.contains(&server_data["gameMode"].as_str().unwrap()) {
            for team in server_data["teams"].as_array().unwrap() {
                let mut team_players: Vec<String> = Vec::new();

                for player in &player_vec {
                    if player.team == *team["name"].as_str().unwrap() && player.state != 5 {
                        team_players.push(player.name.clone());
                    }

                    if player.state == 5 && !spec_vec.contains(&player.name.clone()) {
                        spec_vec.push(player.name.clone());
                    }
                }

                team_vec.push(Team {
                    name: team["name"].as_str().unwrap().to_string(),
                    score: team["score"].as_i64().unwrap(),
                    players: team_players,
                });
            }
        } else {
            for player in &player_vec {
                if player.state != 5 {
                    active_vec.push(player.name.clone());
                } else {
                    spec_vec.push(player.name.clone());
                }
            }
        }

        ctx.send(|m| {
            m.embed(|e| {
                e.colour(0xFF0000);
                e.title(server_data["description"].as_str().unwrap());
                e.url(page_url);
                e.description(embed_desc);

                if TEAMMODES.contains(&server_data["gameMode"].as_str().unwrap()) {
                    for team in team_vec {
                        let mut team_players_display = String::new();
                        for player in team.players {
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
                    for player in active_vec {
                        players_display = format!("{}{}\n", players_display, player);
                    }

                    e.field("Players:", players_display, false);
                }

                if !spec_vec.is_empty() {
                    let mut spec_display = String::new();
                    for player in &spec_vec {
                        if player == &spec_vec[spec_vec.len() - 1] {
                            spec_display = format!("{}{}", spec_display, player);
                        } else {
                            spec_display = format!("{}{}, ", spec_display, player);
                        }
                    }

                    e.field("Spectators:", spec_display, false);
                }

                e.footer(|f| f.text(format!("/connect {ip} {port}")))
            })
        })
        .await?;
    } else {
        // Check if player is in server
        let mut player_stats: Option<Player> = None;
        for player in player_vec {
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
                e.title(server_data["description"].as_str().unwrap());
                e.url(page_url);
                e.description(embed_desc);

                e.field("Frags:", player_stats.as_ref().unwrap().frags, true);
                e.field("Deaths:", player_stats.as_ref().unwrap().deaths, true);
                e.field("KpD:", player_stats.as_ref().unwrap().kpd, true);

                e.field("Accuracy:", player_stats.as_ref().unwrap().acc, true);
                e.field("Flags:", player_stats.as_ref().unwrap().flags, true);
                e.field("Country:", player_stats.as_ref().unwrap().country.as_ref().unwrap_or(&String::from("Unknown")), true);
                e.footer(|f| f.text(format!("/connect {ip} {port}")))
            })
        })
        .await?;
    }

    Ok(())
}

fn server_exists(server_array: &Vec<Value>, host: &String, port: i64) -> bool {
    for server in server_array {
        if server["host"].as_str().unwrap() == host.as_str() && server["port"].as_i64().unwrap() == port {
            return true;
        }
    }

    false
}