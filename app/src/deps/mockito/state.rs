use super::mock::Mock;

pub struct State {
    pub mocks: Vec<Mock>,
}

impl State {
    pub fn new() -> Self {
        Self { mocks: vec![] }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
