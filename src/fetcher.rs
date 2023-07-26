use crate::model::Feed;
use bytes::IntoBuf;
use prost::Message;
use reqwest::Client;

use crate::transit::FeedMessage;

pub async fn fetch(feed: &Feed) -> u32 {
    println!("Fetching {}", feed.name);

    let client = Client::new();

    let headers = feed.to_header_map();
    let response = client
        .get(&feed.url)
        .headers(headers)
        .send()
        .await
        .expect("fetch failed!");
    let bytes = response.bytes().await.unwrap();

    let b = FeedMessage::decode(bytes.into_buf()).unwrap();

    let mut num_trip_updates: u32 = 0;
    for e in b.entity {
        if e.trip_update.is_some() {
            num_trip_updates += 1;
        }
    }
    println!("{}: {} trip updates", feed.name, num_trip_updates);

    num_trip_updates
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Read;
    use std::vec::Vec;

    #[tokio::test]
    async fn test_fetcher() {
        let path = "fixtures/gtfs-07132023-123501";
        let mut buffer: Vec<u8> = Vec::new();
        let mut file = fs::File::open(path).expect("Failed to open the file");
        file.read_to_end(&mut buffer)
            .expect("Failed to read the file");

        let server = mockito::server_address().to_string();
        let mock = mock("GET", "/gtfs")
            .with_status(200)
            .with_body(buffer)
            .create();

        let feed = Feed {
            id: 1,
            name: "Test".to_string(),
            frequency: 5,
            url: format!("http://{}{}", server, "/gtfs"),
            headers: HashMap::new(),
        };

        let num_found = fetch(&feed).await;

        assert_eq!(num_found, 243);

        mock.assert();
    }
}
