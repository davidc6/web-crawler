use mockall::predicate::*;
use mockall::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct DataStoreEntry<T> {
    pub visited: bool,
    pub urls_found: Vec<T>,
}

#[automock]
pub trait DataStore<T, U: 'static>: Debug {
    fn add(&mut self, key: T, value: Option<U>);
    fn visited(&mut self, key: &T);
    fn has_visited(&self, key: &T) -> bool;
    fn exists(&self, key: &T) -> bool;
    fn get<'a>(&'a self, key: &T) -> Option<&'a DataStoreEntry<U>>;
}

#[derive(Debug, PartialEq)]
pub struct Store<T: Hash + Eq + Clone, U> {
    data: HashMap<T, DataStoreEntry<U>>,
}

impl<T: Hash + Eq + Clone, U> Default for Store<T, U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Hash + Eq, U> Store<T, U> {
    pub fn new() -> Self {
        Store {
            data: HashMap::<T, DataStoreEntry<U>>::new(),
        }
    }
}

impl<T: Debug + Clone + Hash + Eq, U: Debug + 'static> DataStore<T, U> for Store<T, U> {
    fn add(&mut self, key: T, value: Option<U>) {
        let item = self.data.get_mut(&key);

        if let Some(item) = item {
            if let Some(value) = value {
                return item.urls_found.push(value);
            }
        }

        self.data.insert(
            key.clone(),
            DataStoreEntry {
                visited: false,
                urls_found: vec![],
            },
        );

        if let Some(value) = value {
            let item = self.data.get_mut(&key);

            if let Some(item) = item {
                item.urls_found.push(value)
            }
        }
    }

    fn exists(&self, key: &T) -> bool {
        if let Some(_val) = self.data.get(key) {
            return true;
        }
        false
    }

    fn get<'a>(&'a self, key: &T) -> Option<&'a DataStoreEntry<U>> {
        self.data.get(key)
    }

    fn visited(&mut self, key: &T) {
        let item = self.data.get_mut(key);

        if let Some(item) = item {
            item.visited = true
        }
    }

    fn has_visited(&self, key: &T) -> bool {
        if let Some(key) = self.data.get(key) {
            return key.visited;
        }
        false
    }
}

#[cfg(test)]
mod data_store_tests {
    use crate::data_store::DataStoreEntry;

    use super::{DataStore, Store};

    #[test]
    fn data_store_adds_key_and_value_correctly() {
        let mut s = Store::new();
        let key = "key".to_owned();
        let val = "val".to_owned();

        s.add(key.clone(), Some(val.clone()));

        assert!(s.exists(&key.clone()));
        assert_eq!(
            s.get(&key),
            Some(&DataStoreEntry {
                visited: false,
                urls_found: vec![val]
            })
        );
    }

    #[test]
    fn data_store_updates_value_correctly() {
        let mut s = Store::new();
        let key = "key".to_owned();
        let val = "val".to_owned();
        let val2 = "val2".to_owned();

        s.add(key.clone(), Some(val.clone()));
        s.add(key.clone(), Some(val2.clone()));

        assert!(s.exists(&key.clone()));
        assert_eq!(
            s.get(&key),
            Some(&DataStoreEntry {
                visited: false,
                urls_found: vec![val, val2]
            })
        );
    }

    #[test]
    fn data_store_adds_key_without_value_correctly() {
        let mut s: Store<String, String> = Store::new();
        let key = "key".to_owned();

        s.add(key.clone(), None);

        assert!(s.exists(&key.clone()));
        assert_eq!(
            s.get(&key),
            Some(&DataStoreEntry {
                visited: false,
                urls_found: vec![]
            })
        );
    }

    #[test]
    fn data_store_set_has_visited_state_to_false_if_not_visited() {
        let mut s = Store::new();
        let key = "key".to_owned();
        let val = Some("val".to_owned());

        s.add(key.clone(), val.clone());

        assert!(!s.has_visited(&key.clone()));
    }

    #[test]
    fn data_store_set_has_visited_state_to_true_if_visited() {
        let mut s = Store::new();
        let key = "key".to_owned();
        let val = Some("val".to_owned());

        s.add(key.clone(), val.clone());
        s.visited(&key);

        assert!(s.has_visited(&key.clone()));
    }
}
