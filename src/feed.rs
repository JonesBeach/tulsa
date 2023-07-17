use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Feed {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub frequency: u32,
}