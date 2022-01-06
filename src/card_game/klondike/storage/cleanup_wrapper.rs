use super::super::Klondike;
use super::klondike_repository::*;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use clokwerk::{Scheduler, TimeUnits, ScheduleHandle};
use std::time::{Duration, Instant};
use std::marker::Send;

/// Wrapper in order to add cleanup to the repository.
/// When used it will delete the stored games after a given period of inactivity
pub struct KlondikeCleanUpRepository<T: KlondikeRepository + Send + 'static, U: TimeoutRepository + Send + 'static> {
    delegate: Arc<Mutex<T>>,
    repo: Arc<Mutex<U>>,
    thread_handle: ScheduleHandle,
}

impl<T: KlondikeRepository + Send + 'static, U: TimeoutRepository + Send + 'static> KlondikeCleanUpRepository<T, U> {
    pub fn new (delegate: T, timeout: Duration, repo: U) -> KlondikeCleanUpRepository<T, U> {

        let delegate = Arc::new(Mutex::new(delegate));
        let repo = Arc::new(Mutex::new(repo));

        let sch_delegate = Arc::clone(&delegate);
        let sch_repo = Arc::clone(&repo);

        let mut scheduler = Scheduler::new();
        scheduler.every(10.seconds()).run (move || {
            let to_remove = { sch_repo.lock().unwrap().get_expired(&timeout) };
            for id in to_remove {
                sch_delegate.lock().unwrap().delete(&id);
            }
        });

        let thread_handle = scheduler.watch_thread(timeout / 10);

        let result = KlondikeCleanUpRepository {
            delegate, 
            repo,
            thread_handle,
        };

        result
    }
}

impl<T: KlondikeRepository + Send + 'static, U: TimeoutRepository + Send + 'static> KlondikeRepository 
        for KlondikeCleanUpRepository< T, U> {

    fn save(&mut self, klondike: Klondike) -> String {
        let result =  { self.delegate.lock().unwrap().save(klondike) };

        self.repo.lock().unwrap().save_last_access(&result);

        result
    }

    fn update(&mut self, id: String, klondike: Klondike) {
        let result = { self.delegate.lock().unwrap().update(id.clone(), klondike) };

        self.repo.lock().unwrap().save_last_access(&id);

        result
    }

    fn get(&self, id: &String) -> Option<Klondike> {
        let result = { self.delegate.lock().unwrap().get(id) };

        if result.is_some() {
            self.repo.lock().unwrap().save_last_access(id);
        }

        result
    }

    fn delete(&mut self, id: &String) -> Option<Klondike> {
        let result = { self.delegate.lock().unwrap().delete(id) };

        self.repo.lock().unwrap().remove(id);

        result
    }
}

/// Storage system for access timestamps.
pub trait TimeoutRepository {

    /// Set the last access time for id to now
    fn save_last_access(&mut self, id: &String);

    /// Returns a vector containing all expired ids
    /// Expired meaning: (now - last_access_time) > timeout
    fn get_expired(&mut self, timeout: &Duration) -> Vec<String>;

    fn remove(&mut self, id: &String);

}

pub struct HashMapTimeoutRepository {
    times: HashMap<String, Instant>,
}

impl HashMapTimeoutRepository {
    pub fn new() -> HashMapTimeoutRepository {
        HashMapTimeoutRepository {times: HashMap::new()}
    }
}

impl TimeoutRepository for HashMapTimeoutRepository {
    fn save_last_access(&mut self, id: &String) {
        self.times.insert(id.clone(), Instant::now());
    }

    fn get_expired(&mut self, timeout: &Duration) -> Vec<String> {
        let mut result = Vec::new();

        for (id, instant) in self.times.iter() {
            if instant.elapsed() > *timeout {
                result.push(id.to_string());
            }
        }
        
        for id in &result {
            self.times.remove(id);
        }
    
        result
    }

    fn remove(&mut self, id: &String) {
        self.times.remove(id);
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::{eq, always};
    use std::thread;

    #[test]
    fn save () {
        let mut delegate = MockKlondikeRepository::new();
        let klondike = Klondike::new();
        delegate.expect_save().with(eq(klondike.clone()))
                .returning(|_x| String::from("xxxx"));

        let mut repo = KlondikeCleanUpRepository::new(delegate, Duration::from_secs(1),
                        HashMapTimeoutRepository::new());
        assert_eq! (repo.save(klondike), String::from("xxxx"));
    }


    #[test]
    fn update() {
        let mut delegate = MockKlondikeRepository::new();
        let klondike = Klondike::new();
        delegate.expect_update()
                .with(eq(String::from("xxxx")), eq(klondike.clone()))
                .returning(|_x, _y| ());

        let mut repo = KlondikeCleanUpRepository::new(delegate, Duration::from_secs(1),
                HashMapTimeoutRepository::new());
        repo.update(String::from("xxxx"), klondike);
    }

    #[test]
    fn get_existing() {
        let id = String::from("testId");
        let mut delegate = MockKlondikeRepository::new();
        let klondike = Klondike::new();
        let klondike_copy = Some(klondike.clone());
        delegate.expect_get().with(eq(id.clone()))
                .return_once(|_x| klondike_copy);

        let repo = KlondikeCleanUpRepository::new(delegate, Duration::from_secs(1), 
                        HashMapTimeoutRepository::new());

        assert_eq! (repo.get(&id), Some(klondike));
    }

    #[test]
    fn get_non_existing() {
        let id = String::from("testId");
        let mut delegate = MockKlondikeRepository::new();
        delegate.expect_get().with(always())
                .return_once(|_x| None);

        let repo = KlondikeCleanUpRepository::new(delegate, Duration::from_secs(1),
                        HashMapTimeoutRepository::new());

        assert_eq! (repo.get(&id), None);
    }

    #[test]
    fn delete_existing() {
        let id = String::from("testId");
        let mut delegate = MockKlondikeRepository::new();
        let klondike = Klondike::new();
        let klondike_copy = Some(klondike.clone());
        delegate.expect_delete().with(eq(id.clone()))
                .return_once(|_x| klondike_copy);

        let mut repo = KlondikeCleanUpRepository::new(delegate, Duration::from_secs(1),
                        HashMapTimeoutRepository::new());

        assert_eq! (repo.delete(&id), Some(klondike));
    }

    #[test]
    fn delete_non_existing() {
        let id = String::from("testId");
        let mut delegate = MockKlondikeRepository::new();
        delegate.expect_delete().with(always())
                .return_once(|_x| None);

        let mut repo = KlondikeCleanUpRepository::new(delegate, Duration::from_secs(1),
                        HashMapTimeoutRepository::new());

        assert_eq! (repo.delete(&id), None);
    }

    #[test]
    fn timeout() {
        let mut delegate = MockKlondikeRepository::new();
        let klondike = Klondike::new();
        let klondike2 = Klondike::new();
        delegate.expect_save().with(eq(klondike.clone()))
                .returning(|_x| String::from("xxxx"));
        delegate.expect_save().with(eq(klondike2.clone()))
                .returning(|_x| String::from("yyyy"));
        delegate.expect_delete().with(eq(String::from("yyyy")))
                .times(1)
                .return_once(|_x| None); //Don't care
        delegate.expect_get().with(always())
                .returning(|_x| None); //Don't care

        let mut repo = KlondikeCleanUpRepository::new(delegate, Duration::from_millis(100), 
                        HashMapTimeoutRepository::new());

        repo.save(klondike);
        repo.save(klondike2);

        let ten_millis = Duration::from_millis(10);
        let id = String::from("xxxx");
        for _i in 0..12 {
            thread::sleep(ten_millis);
            repo.get(&id);
        }

    }
}

