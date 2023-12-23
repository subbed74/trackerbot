use dns_lookup::lookup_host;
use poise::serenity_prelude as serenity;
use poise::Context;
use serde_json::Value;
use serde::Deserialize;

// Data structures
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

#[derive(Default, Deserialize, Debug)]
pub struct Player {
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

pub struct Team {
    pub name: String,
    pub score: i64,
    pub players: Vec<String>,
}

pub struct Server {
    pub description: String,
    pub host: String,
    pub port: i64,
    pub clients: i64,
    pub max_clients: i64,
    pub gamemode: String,
    pub mapname: String,
    pub mastermode: String,
    pub time_left_string: String,
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

pub async fn grab_api_data(client: &reqwest::Client, api_url: String) -> Option<Value> {
    let init_request = client.get(api_url).send().await;

    let init_request = match init_request {
        Ok(ok) => Some(ok),
        Err(err) => {
            if err.is_timeout() {
                println!("Timeout");
            } else {
                println!("{}", err);
            }

            return None;
        }
    };

    let res = init_request.unwrap().json::<serde_json::Value>().await;

    match res {
        Ok(ok) => Some(ok),
        Err(e) => {
            println!("JSON ERROR: {}", e);

            None
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

// Modified paginate code for the user list
pub async fn paginate<U, E>(
    ctx: Context<'_, U, E>,
    title: String,
    pages: &[&str],
    embed_url: Option<String>,
) -> Result<(), serenity::Error> {
    // Weird stuff
    let original_embed_url = embed_url.clone();

    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // Send the embed with the first page as content
    let mut current_page = 0;
    ctx.send(|b| {
        b.embed(|b| {
            b.description(pages[current_page]);

            if original_embed_url.is_some() {
                b.url(original_embed_url.unwrap());
            }

            b.title(&title)
        })
        .components(|b| {
            b.create_action_row(|b| {
                b.create_button(|b| b.custom_id(&prev_button_id).emoji('◀'))
                    .create_button(|b| b.custom_id(&next_button_id).emoji('▶'))
            })
        })
    })
    .await?;

    // Loop through incoming interactions with the navigation buttons
    while let Some(press) = serenity::CollectComponentInteraction::new(ctx)
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
        press
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
            .await?;
    }

    Ok(())
}