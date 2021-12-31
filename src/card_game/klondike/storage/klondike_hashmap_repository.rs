use super::super::Klondike;
use super::klondike_repository::KlondikeRepository;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;


struct KlondikeHashMapRepository {
    games: Mutex<HashMap<String, Klondike>>,
}

/// Simple implementation of KlondikeRepository using Hashmap.
impl KlondikeRepository for KlondikeHashMapRepository {

    fn save(&self, klondike: Klondike) -> String {
        let my_uuid = Uuid::new_v4();
        let uuid = format!("{}", my_uuid);

        self.update(uuid.clone(), klondike);

        uuid
    }

    fn update(&self, id: String, klondike: Klondike) {
        let mut games = self.games.lock().expect("lock shared data");
        
        games.insert(id, klondike);
    }

    fn get(&self, id: &String) -> Option<Klondike> {
        let games = self.games.lock().expect("lock shared data");
        
        games.get(id).map(|x| (*x).clone())
    }

    fn delete(&self, id: &String) -> Option<Klondike> {
        let mut games = self.games.lock().expect("lock shared data");
        
        games.remove(id)
    }
}

impl KlondikeHashMapRepository {
    pub fn new() -> Self {
        KlondikeHashMapRepository {games: Mutex::new(HashMap::new())}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::klondike_repository::test::*;

    #[test]
    fn save_update_get_hashmap() {
        save_update_get(&KlondikeHashMapRepository::new());
    }

    #[test]
    fn delete_hashmap() {
        delete(&KlondikeHashMapRepository::new());
    }
}