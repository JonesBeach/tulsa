use crate::fetcher::transit::FeedMessage;
use prost::Message; // needed for FeedMessage::decode
use reqwest::Client;
use tokio::time::{Duration, Interval};

mod transit {
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}
use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Feed {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub frequency: u64,
    pub headers: HashMap<String, String>,
}

impl Feed {
    pub fn to_header_map(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (key, value) in self.headers.iter() {
            let new_key: HeaderName = key.parse().unwrap();
            headers.insert(new_key, value.parse().unwrap());
        }
        headers
    }
}

async fn fetch(feed: &Feed) -> usize {
    println!("Fetching {}", feed.name);

    let client = Client::new();

    let headers = feed.to_header_map();
    let response = client
        .get(&feed.url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Error fetching {}: {}", feed.name, e);
            e
        }).unwrap();

    let bytes = response.bytes().await
        .map_err(|e| eprintln!("Error reading {}: {}", feed.name, e))
        .unwrap();

    let b = FeedMessage::decode(bytes)
        .map_err(|e| eprintln!("Error decoding {}: {}", feed.name, e))
        .unwrap();

    let mut num_trip_updates: usize = 0;
    for e in b.entity {
        if e.trip_update.is_some() {
            num_trip_updates += 1;
        }
    }
    println!("{}: {} trip updates", feed.name, num_trip_updates);

    num_trip_updates
}

pub async fn recurring_fetch(feed: Feed) {
    let interval_duration = Duration::from_secs(feed.frequency);
    let mut interval: Interval = tokio::time::interval(interval_duration);

    loop {
        interval.tick().await;
        // It might technically be more accurate timer-wise to spawn this
        // like so: tokio::spawn(fetch(feed.clone()));
        fetch(&feed).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Read;
    use std::vec::Vec;

    #[tokio::test]
    async fn test_fetcher() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut file = fs::File::open("fixtures/gtfs-07132023-123501")
            .map_err(|e| eprintln!("Failed to open the file: {}", e))
            .unwrap();
        file.read_to_end(&mut buffer)
            .expect("Failed to read the file");

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/gtfs")
            .with_status(200)
            .with_body(buffer)
            .create();

        let feed = Feed {
            id: 1,
            name: "Test".to_string(),
            frequency: 5,
            url: format!("{}/gtfs", server.url()),
            headers: HashMap::new(),
        };

        let num_found = fetch(&feed).await;

        mock.assert();
        assert_eq!(num_found, 243);
    }
}
