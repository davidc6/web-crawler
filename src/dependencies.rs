use crate::url_frontier::URLFrontier;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Deps<T> = Arc<RwLock<Dependencies<T>>>;

#[derive(Default)]
pub struct DataStore {}

impl<T: Default> Dependencies<T> {
    pub fn new() -> Dependencies<T> {
        Dependencies::default()
    }

    pub fn url_frontier(self, url_frontier: URLFrontier<T>) -> Dependencies<T> {
        let Self { data_store, .. } = self;
        Dependencies {
            url_frontier,
            data_store,
        }
    }

    pub fn data_store(self, data_store: DataStore) -> Dependencies<T> {
        let Self { url_frontier, .. } = self;
        Dependencies {
            url_frontier,
            data_store,
        }
    }

    pub fn build(self) -> Arc<RwLock<Dependencies<T>>> {
        Arc::new(RwLock::new(Self {
            url_frontier: self.url_frontier,
            data_store: self.data_store,
        }))
    }
}

#[derive(Default)]
pub struct Dependencies<T> {
    pub url_frontier: URLFrontier<T>,
    pub data_store: DataStore,
}
