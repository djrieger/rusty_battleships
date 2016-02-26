pub struct ClientLobby {
    player_name: String,
    player_list: Vec<String>,
    feature_list: Vec<String>,
}

impl ClientLobby {

    pub fn new() -> ClientLobby {
        ClientLobby {
            player_name: String::from("Anonymous"),
            player_list: Vec::<String>::new(),
            feature_list: Vec::<String>::new(),
        }
    }

    pub fn get_player_name(&self) -> &str {
        return &self.player_name;
    }

    pub fn set_player_name(&mut self, name: &str ) {
        self.player_name = String::from(name);
    }

    pub fn get_player_list(&self) -> Vec<String> {
        return self.player_list.clone();
    }

    pub fn set_player_list(&mut self, players: Vec<String>) {
        self.player_list = players;
    }

    pub fn set_feature_list(&mut self, features: Vec<String>) {
        self.feature_list = features;
    }
}
