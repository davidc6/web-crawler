use crate::{data_store::Store, url_frontier::URLFrontier};
use std::{fmt::Debug, hash::Hash, sync::Arc};
use tokio::sync::RwLock;

pub type Deps<T, U> = Arc<RwLock<Dependencies<T, U>>>;

#[derive(Default)]
pub struct DataStore {}

impl<T: Clone + Default + Hash + Eq, U: Debug + Default> Dependencies<T, U> {
    pub fn new() -> Dependencies<T, U> {
        Dependencies::default()
    }

    pub fn url_frontier(self, url_frontier: URLFrontier<T>) -> Dependencies<T, U> {
        let Self { data_store, .. } = self;
        Dependencies {
            url_frontier,
            data_store,
        }
    }

    pub fn data_store(self, data_store: Store<T, U>) -> Dependencies<T, U> {
        let Self { url_frontier, .. } = self;
        Dependencies {
            url_frontier,
            data_store,
        }
    }

    pub fn build(self) -> Arc<RwLock<Dependencies<T, U>>> {
        Arc::new(RwLock::new(Self {
            url_frontier: self.url_frontier,
            data_store: self.data_store,
        }))
    }
}

#[derive(Default)]
pub struct Dependencies<T: Clone + Hash + Eq, U> {
    pub url_frontier: URLFrontier<T>,
    pub data_store: Store<T, U>,
}
