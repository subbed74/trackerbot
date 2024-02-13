#![allow(non_snake_case)] // Just here to align with the JSON when needed for my sanity

use dns_lookup::lookup_host;
use poise::serenity_prelude as serenity;
use poise::Context;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use crate::Error;

// Data structures
#[allow(dead_code)]
pub const MODENAMES: [&str; 23] = [
    "ffa",
    "coop_edit",
    "teamplay",
    "instagib",
    "insta_team",
    "efficiency",
    "effic_team",
    "tactics",
    "tac_team",
    "capture",
    "regen_capture",
    "ctf",
    "insta_ctf",
    "protect",
    "insta_protect",
    "hold",
    "insta_hold",
    "effic_ctf",
    "effic_protect",
    "effic_hold",
    "collect",
    "insta_collect",
    "effic_collect",
];

pub const TEAMMODES: [&str; 18] = [
    "teamplay",
    "insta_team",
    "effic_team",
    "tac_team",
    "capture",
    "regen_capture",
    "ctf",
    "insta_ctf",
    "protect",
    "insta_protect",
    "hold",
    "insta_hold",
    "effic_ctf",
    "effic_protect",
    "effic_hold",
    "collect",
    "insta_collect",
    "effic_collect",
];

#[derive(Clone, Default, Deserialize, Debug)]
pub struct BasicServer {  // Used for the server list
    pub descriptionStyled: String,
    pub description: String,
    pub country: String,
    pub countryName: String,
    pub host: String,
    pub port: i64,
    pub version: i64,
    pub clients: i64,
    pub maxClients: i64,
    pub gameMode: String,
    pub mapName: String,
    pub masterMode: String,
    pub isFull: bool,
    pub timeLeft: i64,
    pub timeLeftString: String,
    pub zombie: bool,
    pub players: Vec<String>
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct DetailedServer { // Used for more detailed information
    pub descriptionStyled: String,
    pub description: String,
    pub country: String,
    pub countryName: String,
    pub host: String,
    pub port: i64,
    pub version: i64,
    pub info: Info,
    pub clients: i64,
    pub maxClients: i64,
    pub gameMode: String,
    pub mapName: String,
    pub masterMode: String,
    pub isFull: bool,
    pub timeLeft: i64,
    pub timeLeftString: String,
    pub zombie: bool,
    pub players: Vec<ServerPlayer>,
    pub teams: Vec<Team>,
    pub gameType: String,

    // Non-JSON provided data
    pub all_active_players: Option<Vec<String>>,
    pub spectators: Option<Vec<String>>
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerPlayer {
    pub name: String,
    pub frags: i64,
    pub team: String,
    pub flags: i64,
    pub deaths: i64,
    pub kpd: f64,
    pub acc: i64,
    pub tks: i64,
    pub state: i64,
    pub country: Option<String>,
    pub ping: i64,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Team {
    pub name: String,
    pub score: i64,

    // Non-JSON provided data
    pub players: Option<Vec<String>>
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Info {
    pub website: String,
    pub demourl: String,
    pub banned: String,
}

// DB specific structs
pub struct ServerBookmark {
    pub id: i32,
    pub guild_id: u64,
    pub bookmark_name: String,
    pub host: String,
    pub port: u32,
}

// API handling
pub async fn resolve_ip(initial: String) -> Option<String> {
    match lookup_host(&initial) {
        Ok(ok) => Some(ok[0].to_string()),
        Err(err) => {
            println!("{err}");
            None
        }
    }
}

pub async fn grab_api_data(client: &reqwest::Client, api_url: String, backup_url: &String) -> Result<Value, Error> {
    let init_request = client.get(&api_url).send().await;

    let init_request = match init_request {
        Ok(ok) => ok,
        Err(err) => {
            if err.is_timeout() {
                println!("[ ERROR ] Bot timed out using api_url: {}", &api_url);
                return Err(format!("The request has timed out! Try visiting: {backup_url}").into());
            } else {
                println!("{err}");
            }

            return Err("There was an unexpected error with the request! Try visiting: {backup_url}".into());
        }
    };

    let res = init_request.json::<serde_json::Value>().await;

    match res {
        Ok(ok) => Ok(ok),
        Err(e) => {
            println!("[ ERROR ] An error occured grabbing JSON data: {e}");

            Err("There was an unexpected error!".into())
        }
    }
}

// Don't format for Discord markdown
pub fn escape_markdown(mut text: String) -> String {
    text = text.replace('*', "\\*");
    text = text.replace('_', "\\_");
    text = text.replace('|', "\\|");
    text = text.replace('#', "\\#");

    text
}

// Modified sample paginate code for the user list
pub async fn paginate<U, E>(
    ctx: Context<'_, U, E>,
    title: String,
    pages: &[&str],
    embed_url: Option<String>,
) -> Result<(), serenity::Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // Send the embed with the first page as content
    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&prev_button_id).emoji('◀'),
            serenity::CreateButton::new(&next_button_id).emoji('▶'),
        ]);

        poise::CreateReply::default()
            .embed(serenity::CreateEmbed::default().description(pages[0]).title(&title).url(embed_url.clone().unwrap_or_else(String::new)))
            .components(vec![components])
    };

    ctx.send(reply).await?;

    // Send the embed with the first page as content
    let mut current_page = 0;

    // Loop through incoming interactions with the navigation buttons
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        /*press
            .create_interaction_response(ctx, |b| {
                b.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| {
                        b.embed(|b| {
                            b.description(pages[current_page]);

                            if embed_url.clone().is_some() {
                                b.url(embed_url.clone().unwrap());
                            }

                            b.title(&title)
                        })
                    })
            })
            .await?;*/

        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(serenity::CreateEmbed::new().description(pages[current_page]).title(&title).url(embed_url.clone().unwrap_or_else(String::new))),
                ),
            )
            .await?;
    }

    Ok(())
}