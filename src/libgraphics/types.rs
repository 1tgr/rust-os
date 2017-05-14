#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    CreateWindow,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    KeyPress(char),
}
