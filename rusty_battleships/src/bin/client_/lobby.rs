use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ClientLobby {
    pub player_name: String,
    // player name -> is ready 
    player_list: HashMap<String, bool>,
    pub feature_list: Vec<String>,
}

impl ClientLobby {
    pub fn new() -> ClientLobby {
        ClientLobby {
            player_name: String::from("Anonymous"),
            player_list: HashMap::new(),
            feature_list: vec![],
        }
    }

    pub fn set_player_name(&mut self, name: &str ) {
        self.player_name = String::from(name);
    }

    pub fn add_player(&mut self, player: &str) {
        self.player_list.insert(String::from(player), false);
    }

    pub fn remove_player(&mut self, player: &str) {
        self.player_list.remove(player).expect(&format!("Tried removing non-existing player {}", player));
    }

    fn change_ready_state(&mut self, player: &str, ready: bool) {
        let entry = self.player_list
            .get_mut(&String::from(player))
            .expect(&format!("Tried setting ready state to {} for non-existing player {}", ready, player));
        *entry = ready; 
    }

    pub fn get_available_players(&self) -> Vec<String> {
        self.player_list
            .keys()
            .map(|ref name| (**name).clone())
            .collect()
    }

    pub fn get_ready_players(&self) -> Vec<String> {
        self.player_list
            .iter()
            .filter(|&(_, is_ready)| *is_ready == true)
            .map(|(ref name, _)| (**name).clone())
            .collect()
    }

    pub fn ready_player(&mut self, player: &str) {
        self.change_ready_state(player, true);
    }

    pub fn unready_player(&mut self, player: &str) {
        self.change_ready_state(player, false);
    }

    pub fn set_feature_list(&mut self, features: Vec<String>) {
        self.feature_list = features;
    }
}
