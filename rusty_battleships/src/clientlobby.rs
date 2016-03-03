#[derive(Clone, Debug)]
pub struct ClientLobby {
    //these are public for avoiding the need for getters and setters.
    pub player_name: String,
    pub player_list: Vec<String>,
    pub ready_players: Vec<String>,
    pub feature_list: Vec<String>,
}

impl ClientLobby {

    pub fn new() -> ClientLobby {
        ClientLobby {
            player_name: String::from("Anonymous"),
            player_list: Vec::<String>::new(),
            ready_players: Vec::<String>::new(),
            feature_list: Vec::<String>::new(),
        }
    }

    pub fn set_player_name(&mut self, name: &str ) {
        self.player_name = String::from(name);
    }

    pub fn set_player_list(&mut self, players: Vec<String>) {
        self.player_list = players;
    }

    pub fn add_player(&mut self, player: &str) {
        self.player_list.push(String::from(player));
    }

    pub fn ready_player(&mut self, player: &str) {
        if self.player_list.contains(&String::from(player)) {
            //This is safe because we already ensured that the element is contained.
            let index = self.player_list.binary_search(&String::from(player)).unwrap();
            self.player_list.remove(index);
            self.ready_players.push(String::from(player));
        } else {
            panic!("Well, fuck. Got a PLAYER_READY_UPDATE for a player who's not in our list.");
        }
    }

    pub fn set_feature_list(&mut self, features: Vec<String>) {
        self.feature_list = features;
    }
}
