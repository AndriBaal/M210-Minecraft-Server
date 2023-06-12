use actix_files::Files;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_websockets::{Event, Message};
use simple_websockets::{EventHub, Responder as Client};
use std::ops::DerefMut;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

const INVALID_NAME: String = String::new();
const GAME_SIZE: (i32, i32, i32) = (50, 10, 50);
const START: i32 = -40;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Team {
    Blue,
    Red,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    name: String,
    #[serde(default)]
    running: bool,
    #[serde(default)]
    red_players: HashMap<String, Player>,
    #[serde(default)]
    blue_players: HashMap<String, Player>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Player {
    game: String,
    name: String,
    team: Team,
}

struct AppState {
    games: HashMap<String, Game>,
    handlebars: Handlebars<'static>,
    _hub: EventHub,
    client: Client,
    game_counter: i32,
}

fn send(hub: &EventHub, client: &mut Client, command: String) {
    let msg = json!({
      "header": {
        "version": 1,
        "requestId": Uuid::new_v4(),
        "messagePurpose": "commandRequest",
        "messageType": "commandRequest"
      },
      "body": {
        "version": 1,
        "commandLine": command
      }
    });

    if !client.send(Message::Text(msg.to_string())) {
        loop {
            match hub.poll_event() {
                Event::Connect(_, responder) => {
                    *client = responder;
                    break;
                }
                _ => {}
            }
        }
    }
}

#[get("/")]
async fn index(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let state = data.lock().unwrap();
    let view = state
        .handlebars
        .render("template", &HashMap::from([("games", &state.games)]))
        .unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(view)
}

#[get("/update_game")]
async fn update_game(
    params: web::Query<HashMap<String, String>>,
    data: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let mut state = data.lock().unwrap();
    let state_ref = state.deref_mut();
    if let Some(game) = state_ref
        .games
        .get_mut(params.get("game").unwrap_or(&INVALID_NAME))
    {
        game.running = !game.running;
        if game.running {
            state_ref.game_counter += 1;
            let offset = START + state_ref.game_counter % 20 * GAME_SIZE.1;

            let x1 = GAME_SIZE.0 / 2;
            let x2 = -GAME_SIZE.0 / 2;
            let z1 = -GAME_SIZE.2 / 2;
            let z2 = GAME_SIZE.2 / 2;

            send(
                &state_ref._hub,
                &mut state_ref.client,
                format!(
                    "fill {} {} {} {} {} {} bedrock hollow",
                    x1,
                    offset,
                    z1,
                    x2,
                    offset + GAME_SIZE.1,
                    z2
                ),
            );

            for (_, player) in game.blue_players.iter().chain(game.red_players.iter()) {
                let (_color_name, x_offset) = match &player.team {
                    Team::Blue => ("Blue", 10.0),
                    Team::Red => ("Red", -10.0),
                };

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/gamerule keepinventory true"),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!(
                        "/replaceitem entity {} slot.armor.head 1 iron_helmet",
                        player.name
                    ),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/clear {}", player.name),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/effect {} clear", player.name),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/effect {} instant_health 1 4", player.name),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/effect {} night_vision 100000 1", player.name),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/scoreboard teams add {} ", player.name),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!(
                        "/replaceitem entity {} slot.armor.head 1 iron_helmet",
                        player.name
                    ),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!(
                        "/replaceitem entity {} slot.armor.chest 1 iron_chestplate",
                        player.name
                    ),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!(
                        "/replaceitem entity {} slot.armor.legs 1 iron_leggings",
                        player.name
                    ),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!(
                        "/replaceitem entity {} slot.armor.feet 1 iron_boots",
                        player.name
                    ),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/give {} iron_sword 1", player.name),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/give {} apple 16", player.name),
                );

                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/teleport {} {} {} 0", player.name, x_offset, offset + 3),
                );
            }
        } else {
            for (_, player) in game.blue_players.iter().chain(game.red_players.iter()) {
                send(
                    &state_ref._hub,
                    &mut state_ref.client,
                    format!("/effect {} weakness 100000 255", player.name),
                );
            }
        }
    }
    return web::Redirect::to("/");
}

#[get("/remove_game")]
async fn remove_game(
    params: web::Query<HashMap<String, String>>,
    data: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let mut state = data.lock().unwrap();
    state
        .games
        .remove(params.get("game").unwrap_or(&INVALID_NAME));
    return web::Redirect::to("/");
}

#[get("/remove_player")]
async fn remove_player(
    params: web::Query<HashMap<String, String>>,
    data: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let mut state = data.lock().unwrap();
    if let Some(game) = state
        .games
        .get_mut(params.get("game").unwrap_or(&INVALID_NAME))
    {
        if let Some(player_name) = params.get("player") {
            game.red_players.remove(player_name);
            game.blue_players.remove(player_name);
        }
    }
    return web::Redirect::to("/");
}

#[get("/add_game")]
async fn add_game(game: web::Query<Game>, data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    if game.name.len() > 0 {
        let mut state = data.lock().unwrap();
        state.games.insert(game.name.clone(), game.0);
        drop(state);
    }
    return web::Redirect::to("/");
}

#[get("/add_player")]
async fn add_player(
    player: web::Query<Player>,
    data: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    let mut state = data.lock().unwrap();
    if let Some(game) = state.games.get_mut(&player.game) {
        if !game.blue_players.contains_key(&player.name)
            && !game.red_players.contains_key(&player.name)
            && player.name.len() > 0
        {
            match player.team {
                Team::Blue => game.blue_players.insert(player.name.clone(), player.0),
                Team::Red => game.red_players.insert(player.name.clone(), player.0),
            };
        }
    }
    return web::Redirect::to("/");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let hub = simple_websockets::launch(3000).unwrap();
    let client;
    loop {
        match hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}", client_id);
                client = responder;
                break;
            }
            _ => {}
        }
    }

    let state = Arc::new(Mutex::new(AppState {
        games: Default::default(),
        handlebars: {
            let mut handlebars = Handlebars::new();
            handlebars
                .register_template_file("template", "./views/template.hbs")
                .unwrap();
            handlebars
        },
        client,
        game_counter: 0,
        _hub: hub,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(Files::new("/static", "./static").show_files_listing())
            .service(index)
            .service(add_game)
            .service(add_player)
            .service(remove_game)
            .service(update_game)
            .service(remove_player)
    })
    .bind(("0.0.0.0", 8069))?
    .run()
    .await
}
