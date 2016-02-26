#[derive(Clone, Debug)]
pub struct ClientLobby {
    //these are public for avoiding the need for getters and setters.
    pub player_name: String,
    pub player_list: Vec<String>,
    pub feature_list: Vec<String>,
}

impl ClientLobby {

    pub fn new() -> ClientLobby {
        ClientLobby {
            player_name: String::from("Anonymous"),
            player_list: Vec::<String>::new(),
            feature_list: Vec::<String>::new(),
        }
    }

    pub fn set_player_name(&mut self, name: &str ) {
        self.player_name = String::from(name);
    }

    pub fn set_player_list(&mut self, players: Vec<String>) {
        self.player_list = players;
    }

    pub fn set_feature_list(&mut self, features: Vec<String>) {
        self.feature_list = features;
    }
}
