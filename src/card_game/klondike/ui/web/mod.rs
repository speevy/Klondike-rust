use uuid::Uuid;
use std::collections::HashMap;
use crate::card_game::klondike::*;
use rocket::{State, Error, response};
use rocket::response::{Responder, Response};
use rocket::request::Request;
use rocket::http::{ContentType, Header, Status};
use std::sync::Mutex;
use rocket::serde::json::Json;
use crate::card_game::klondike::ui::get_card_holder;
use serde::{Serialize, Deserialize};

use clokwerk::{Scheduler, TimeUnits};
use std::time::{Duration, Instant};

struct KlondikeGames {
    games: &'static Mutex<GameInfo>,
}

struct GameInfo {
    games: HashMap<String, Klondike>,
    last_access: HashMap<String, Instant>,
}

#[derive(Deserialize)]
struct Action {
    action: String,
    from: Option<String>,
    to: Option<String>,
    number: Option<u32>
}

#[derive(Responder)]
#[response(status = 201)]
struct Created<T> {
    inner: T,
    header: Header<'static>
}

impl Created<()> {
    fn new(url: String) -> Created<()> {
        Created {header: Header::new("Location", url), inner: ()}
    }
}

#[post("/game")]
fn new_game(shared: &State<KlondikeGames>) -> Created<()> {
    let my_uuid = Uuid::new_v4();
    let uuid = format!("{}", my_uuid);
    let mut state = shared.games.lock().expect("lock shared data");
    state.games.insert(uuid.clone(), Klondike::new());
    state.last_access.insert(uuid.clone(), Instant::now());

    return Created::new(format!("/klondike/game/{}", uuid));
}

#[get("/game/<uuid>")]
fn get_status(uuid: String, shared: &State<KlondikeGames>) 
            -> ApiResponse<Option<KlondikeStatus>> {

    execute(uuid, shared, |_x: &mut Klondike| -> Status { Status::Ok })

}

#[put("/game/<uuid>", data="<action>")]
fn execute_action(uuid: String, action: Json<Action>, shared: &State<KlondikeGames>) 
            ->  ApiResponse<Option<KlondikeStatus>> {

    execute(uuid, shared, |x: &mut Klondike| -> Status {
        match action.action.as_str() {
            "take" => { x.take(); return Status::Ok },
            "undo" => { x.undo(); return Status::Ok },
            "move" => {
                let from_o_ch = get_card_holder(action.from.as_ref().map(|x| x.as_str()));
                let to_o_ch = get_card_holder(action.to.as_ref().map(|x| x.as_str()));
                if let (Some(from_ch), Some(to_ch)) = (from_o_ch, to_o_ch) {
                    if x.move_cards(from_ch, to_ch, action.number.unwrap_or(1)) {
                        return Status::Ok;
                    } else {
                        return Status::Forbidden;
                    }
                } else {
                    return Status::BadRequest;
                }
            },
            _ => Status::BadRequest
        }
    })
}

#[delete("/game/<uuid>")]
fn delete(uuid: String, shared: &State<KlondikeGames>) -> Status {
    let mut state = shared.games.lock().expect("lock shared data");
    
    state.last_access.remove(&uuid);

    match state.games.remove(&uuid) {
        Some(_x) => Status::Ok,
        None => Status::NotFound
    }
}

fn execute<F: Fn(&mut Klondike) -> Status>(
            uuid: String, 
            shared: &State<KlondikeGames>, 
            task: F) -> ApiResponse<Option<KlondikeStatus>> {

    let mut state = shared.games.lock().expect("lock shared data");

    if let Some(_x) = state.last_access.get(&uuid) {
        state.last_access.insert(uuid.clone(), Instant::now());
    }

    if let Some(x) = state.games.get_mut(&uuid) {
        let task_result = task(x);
        return ApiResponse { status: task_result, json: Json(Option::Some(x.get_status()))};
    }     
    
    ApiResponse { status: Status::NotFound, json: Json(Option::None)}
}


//Adapted to rocket 0.5 from https://stackoverflow.com/questions/54865824/return-json-with-an-http-status-other-than-200-in-rocket
#[derive(Debug)]
struct ApiResponse<T: Serialize> {
    json: Json<T>,
    status: Status,
}

impl<'r, 'o: 'r, T: Serialize> Responder<'r, 'o> for ApiResponse<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

fn cleanup (games: &Mutex<GameInfo>) {
    let mut state = games.lock().expect("lock shared data");

    let mut to_remove: Vec<String> = Vec::new();
    for (uuid, instant) in state.last_access.iter() {
        if instant.elapsed() > *CLEANUP_TIMEOUT {
            to_remove.push(uuid.to_string());
        }
    }

    println!("Cleanup: games: {} UUIDs to remove {:?}",
        state.games.len(), &to_remove);

    for uuid in to_remove {
        state.games.remove(&uuid);
        state.last_access.remove(&uuid);
    }
}

lazy_static! {
    static ref GAMES:Mutex<GameInfo> = Mutex::new(GameInfo {games: HashMap::new(), last_access: HashMap::new()});
    static ref CLEANUP_TIMEOUT:Duration = Duration::from_secs(15 * 60); // 15 Minutes
}

#[rocket::main]
pub async fn main_rocket() -> Result<(), Error> {
    let state = KlondikeGames { games : &*GAMES };
    let rock = rocket::build()
        .mount("/klondike", routes![new_game, get_status, execute_action, delete])
        .manage(state);

    let mut scheduler = Scheduler::new();
    scheduler.every(10.seconds()).run (|| cleanup(&*GAMES));
    let _thread_handle = scheduler.watch_thread(Duration::from_millis(100));

    rock.launch().await
}
