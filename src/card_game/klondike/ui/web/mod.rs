use uuid::Uuid;
use std::collections::HashMap;
use crate::card_game::klondike::*;
use rocket::{State, Error};
use std::sync::Mutex;
use rocket::serde::json::Json;
use crate::card_game::klondike::ui::get_card_holder;

struct KlondikeGames {
    games: Mutex<HashMap<String, Klondike>>
}

#[get("/new")]
fn new_game(shared: &State<KlondikeGames>) -> String {
    let my_uuid = Uuid::new_v4();
    let mut state = shared.games.lock().expect("lock shared data");
    state.insert(format!("{}", my_uuid), Klondike::new());

    return format!("{}", my_uuid);
}

#[get("/game/<uuid>/status")]
fn status(uuid: String, shared: &State<KlondikeGames>) 
            -> Option<Json<KlondikeStatus>> {
    execute(uuid, shared, |_x: &mut Klondike| -> () {})
}

#[get("/game/<uuid>/take")]
fn take(uuid: String, shared: &State<KlondikeGames>) 
            ->  Option<Json<KlondikeStatus>> {
    execute(uuid, shared, |x| -> () {x.take();})
}

#[get("/game/<uuid>/undo")]
fn undo(uuid: String, shared: &State<KlondikeGames>) 
            ->  Option<Json<KlondikeStatus>> {
    execute(uuid, shared, |x| -> () {x.undo();})
}

#[get("/game/<uuid>/move?<from>&<to>&<number>")]
fn move_cards(uuid: String, from: String, to: String, number: Option<u32>,
    shared: &State<KlondikeGames>) ->  Option<Json<KlondikeStatus>> {

    let from_o_ch = get_card_holder(Some(from.as_str()));
    let to_o_ch = get_card_holder(Some(to.as_str()));
    if let Some(from_ch) = from_o_ch {
        if let Some(to_ch) = to_o_ch {
            return execute(uuid, shared, |x| -> () {
                x.move_cards(from_ch, to_ch, number.unwrap_or(1));
            });
        }
    }

    Option::None
}


fn execute<F: Fn(&mut Klondike)>(
            uuid: String, 
            shared: &State<KlondikeGames>, 
            task: F) -> Option<Json<KlondikeStatus>> {

    let mut state = shared.games.lock().expect("lock shared data");
    let klondike = state.get_mut(&uuid);

    if let Some(x) = klondike {
        task(x);
        return Option::Some(Json(x.get_status()));
    }     
    
    Option::None
}

#[rocket::main]
pub async fn main_rocket() -> Result<(), Error> {
    let state = KlondikeGames
 { games : Mutex::new(HashMap::new()) };
    rocket::build()
        .mount("/klondike", routes![new_game, status, take, undo, move_cards])
        .manage(state)
        .launch()
        .await
}
