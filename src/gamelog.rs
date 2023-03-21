use shipyard::Unique;

#[derive(Debug, Unique)]
pub struct GameLog {
    pub messages: Vec<String>
}
