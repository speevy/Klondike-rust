use super::super::Klondike;
use super::klondike_repository::KlondikeRepository;
use std::collections::HashMap;
use uuid::Uuid;


pub struct KlondikeHashMapRepository {
    games: HashMap<String, Klondike>,
}

/// Simple implementation of KlondikeRepository using Hashmap.
impl KlondikeRepository for KlondikeHashMapRepository {

    fn save(&mut self, klondike: Klondike) -> String {
        let my_uuid = Uuid::new_v4();
        let uuid = format!("{}", my_uuid);

        self.update(uuid.clone(), klondike);

        uuid
    }

    fn update(&mut self, id: String, klondike: Klondike) {
        self.games.insert(id, klondike);
    }

    fn get(&self, id: &String) -> Option<Klondike> {
        self.games.get(id).map(|x| (*x).clone())
    }

    fn delete(&mut self, id: &String) -> Option<Klondike> {
        self.games.remove(id)
    }
}

impl KlondikeHashMapRepository {
    pub fn new() -> Self {
        KlondikeHashMapRepository { games: HashMap::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::klondike_repository::test::*;

    #[test]
    fn save_update_get_hashmap() {
        save_update_get(&mut KlondikeHashMapRepository::new());
    }

    #[test]
    fn delete_hashmap() {
        delete(&mut KlondikeHashMapRepository::new());
    }
}