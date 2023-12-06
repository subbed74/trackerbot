//use poise::serenity_prelude as serenity;
use crate::{Context, Error};
use trackerbot::{grab_api_data, resolve_ip, ActivePlayer, Server, Team, TEAMMODES};

/// Grab active servers.
#[poise::command(slash_command)]
pub async fn listservers(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    // Grab data
    let server_data = match grab_api_data(
        &ctx.data().client,
        String::from("https://sauertracker.net/api/v2/servers"),
    )
    .await
    {
        Some(data) => data,
        None => return Err("Unable to retrieve data!".into()),
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
#[poise::command(slash_command)]
pub async fn server(
    ctx: Context<'_>,
    #[description = "Server Addr"] ip: String,
    #[description = "Server Port"] port: Option<u16>,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Validate args
    let ip = match resolve_ip(ip).await {
        Some(ip) => ip,
        None => return Err("Unable to resolve address!".into()),
    };

    let port = port.unwrap_or(28785_u16);

    // Begin data handling
    let server_data = match grab_api_data(
        &ctx.data().client,
        format!("http://sauertracker.net/api/v2/server/{}/{}", ip, port),
    )
    .await
    {
        Some(data) => data,
        None => return Err("Unable to retrieve data!".into()),
    };

    let mut player_vec: Vec<ActivePlayer> = Vec::new();
    for player in server_data["players"].as_array().unwrap() {
        player_vec.push(ActivePlayer {
            name: player["name"].as_str().unwrap().to_string(),
            team: player["team"].as_str().unwrap().to_string(),
            state: player["state"].as_i64().unwrap(),
        });
    }

    // Format server info
    let embed_desc = format!(
        "**Players:** {}/{} \n **Mastermode:** {} \n *{} {} {}* \n ",
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
            e.url(format!("https://sauertracker.net/server/{}/{}", ip, port));
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

    Ok(())
}
