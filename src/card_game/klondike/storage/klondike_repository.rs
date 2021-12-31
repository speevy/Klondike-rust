use mockall::automock;
use super::super::Klondike;

/// Implementations of storage systems for Klondike games 
/// should implement this trait.
#[automock]
pub trait KlondikeRepository {

    /// Saves the current state and returns the created id for it
    fn save(&self, klondike: Klondike) -> String;

    /// Saves the current state of an already saved game
    fn update(&self, id: String, klondike: Klondike);

    /// Gets a saved game by it's id.
    fn get(&self, id: &String) -> Option<Klondike>;

    /// Removes a saved game from the repository by it's id.
    /// Returns the removed element
    fn delete(&self, id: &String) -> Option<Klondike>;

}

/// Test that should be passed by any implementation of KlondikeRepository
pub mod test {
    use super::*;

    //    pub fn save_update_get<T: KlondikeRepository>(repo: &mut T) {
    pub fn save_update_get(repo: &dyn KlondikeRepository) {
        
        let klondike1 = Klondike::new();
        let status1 = klondike1.get_status();
        let id1 = repo.save(klondike1.clone());

        let klondike2 = Klondike::new();
        let status2 = klondike2.get_status();
        let id2 = repo.save(klondike2);

        let get1 = repo.get(&id1);
        assert_eq!(get1.map(|x| x.get_status()), Some(status1));
        
        let klondike3 = Klondike::new();
        let status3 = klondike3.get_status();
        repo.update(id1.clone(), klondike3);
        let get3 = repo.get(&id1);
        assert_eq!(get3.map(|x| x.get_status()), Some(status3));

        let get2 = repo.get(&id2);
        assert_eq!(get2.map(|x| x.get_status()), Some(status2));

        assert!(repo.get(&String::from("invalid id")).is_none());
    }

    pub fn delete(repo: &dyn KlondikeRepository) {
        let klondike1 = Klondike::new();
        let status1 = klondike1.get_status();
        let id1 = repo.save(klondike1.clone());

        let klondike2 = Klondike::new();
        let status2 = klondike2.get_status();
        let id2 = repo.save(klondike2);

        assert!(repo.get(&String::from("invalid id")).is_none());

        let get1 = repo.delete(&id1);
        assert_eq!(get1.map(|x| x.get_status()), Some(status1));

        assert!(repo.delete(&id1).is_none());
       
        let get2 = repo.get(&id2);
        assert_eq!(get2.map(|x| x.get_status()), Some(status2));        
    }

}