use axum::{
    routing::{get,post},
    http::StatusCode,
    Json,
    Router
};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::VecDeque;

pub mod transit {
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

lazy_static! {
    static ref ID: Mutex<u32> = Mutex::new(1_u32);
    static ref QUEUE: Mutex<VecDeque<Feed>> = Mutex::new(VecDeque::new());
}

#[tokio::main]
async fn main() {
    let feed = transit::FeedMessage::default();
    dbg!(feed);

    let app = Router::new()
        .route("/", get(status_handler))
        .route("/feed", post(feed_post_handler));

    let address: &str = "0.0.0.0:3000";
    println!("Starting server on {}.", address);
    axum::Server::bind(&address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize)]
struct Status {
    status: String,
}

async fn status_handler() -> Json<Status> {
    Json(Status{
        status: "Ok".to_string()
    })
}

#[derive(Serialize, Clone)]
struct Feed {
    id: u32,
    name: String,
    url: String,
    frequency: u32,
}

#[derive(Deserialize)]
struct CreateFeed {
    name: String,
    url: String,
    frequency: u32,
}

#[axum_macros::debug_handler]
async fn feed_post_handler(
    Json(payload): Json<CreateFeed>,
    ) -> (StatusCode, Json<Feed>) {
    let feed = Feed {
        id: *(ID.lock().unwrap()),
        name: payload.name,
        url: payload.url,
        frequency: payload.frequency
    };

    *(ID.lock().unwrap()) += 1;

    QUEUE.lock().unwrap().push_back(feed.clone());

    (StatusCode::CREATED, Json(feed))
}
