use crate::card_game::klondike::*;
use rocket::{State, Error, response};
use rocket::response::{Responder, Response};
use rocket::request::Request;
use rocket::http::{ContentType, Header, Status};
use rocket::fairing::{Fairing, Info, Kind};
use std::sync::{Mutex, Arc};
use rocket::serde::json::Json;
use crate::card_game::klondike::ui::get_card_holder;
use serde::{Serialize, Deserialize};
use crate::card_game::klondike::storage::cleanup_wrapper::{HashMapTimeoutRepository, KlondikeCleanUpRepository};
use crate::card_game::klondike::storage::hashmap_repository::KlondikeHashMapRepository;
use crate::card_game::klondike::storage::klondike_repository::KlondikeRepository;
use std::time::Duration;

struct KlondikeGames {
    repo: Arc<Mutex<dyn KlondikeRepository + Send + 'static>>,
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
    location: Header<'static>,
    expose_location: Header<'static>,
}

impl Created<()> {
    fn new(url: String) -> Created<()> {
        Created {
            location: Header::new("Location", url), 
            expose_location: Header::new("Access-Control-Expose-Headers", "Location"),
            inner: ()
        }
    }
}

#[post("/game")]
fn new_game(shared: &State<KlondikeGames>) -> Created<()> {
    let mut state = shared.repo.lock().unwrap();
    let id = state.save(Klondike::new());

    return Created::new(format!("/klondike/game/{}", id));
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

#[delete("/game/<id>")]
fn delete(id: String, shared: &State<KlondikeGames>) -> Status {
    let mut repo = shared.repo.lock().unwrap();

    match repo.delete(&id) {
        Some(_x) => Status::Ok,
        None => Status::NotFound
    }
}

#[options("/game/<id>")]
fn options(id: String, shared: &State<KlondikeGames>) -> Status {
    let repo = shared.repo.lock().unwrap();

    match repo.get(&id) {
        Some(_x) => Status::Ok,
        None => Status::NotFound
    }
}

fn execute<F: Fn(&mut Klondike) -> Status>(
            id: String, 
            shared: &State<KlondikeGames>, 
            task: F) -> ApiResponse<Option<KlondikeStatus>> {

    let mut repo = shared.repo.lock().unwrap();

    if let Some(x) = repo.get(&id).as_mut() {
        let task_result = task(x);
        repo.update(id, x.clone());
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

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
       }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PUT, PATCH, OPTIONS, DELETE"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[rocket::main]
pub async fn main_rocket() -> Result<(), Error> {
    //TODO: Make repository choices configurable
    let repo = KlondikeHashMapRepository::new();
    let repo = KlondikeCleanUpRepository::new(
        repo, 
        Duration::from_secs(15 * 60), // 15 Minutes
        HashMapTimeoutRepository::new()
    );

    let state = KlondikeGames { repo: Arc::new(Mutex::new(repo))};

    rocket::build()
        .attach(CORS)
        .mount("/klondike", routes![new_game, get_status, execute_action, delete, options])
        .manage(state).launch().await
}
