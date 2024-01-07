use crate::{
    data_store::{DataStore, Store},
    url_frontier::{Queue, URLFrontier},
};
use std::{fmt::Debug, hash::Hash, sync::Arc};
use tokio::sync::RwLock;

pub type Deps<T, U> = Arc<RwLock<Dependencies<T, U>>>;
pub type DepsConcrete = Arc<Dependencies<String, String>>;

impl<
        T: Clone + Default + Debug + Hash + Eq + Send + Sync + 'static,
        U: Send + Debug + Sync + Default + 'static,
    > Dependencies<T, U>
{
    pub fn new() -> Dependencies<T, U> {
        Dependencies {
            url_frontier: Arc::new(RwLock::new(URLFrontier::default())),
            data_store: Arc::new(RwLock::new(Store::default())),
        }
    }

    pub fn url_frontier(
        self,
        url_frontier: Arc<RwLock<dyn Queue<T> + Send + Sync>>,
    ) -> Dependencies<T, U> {
        let Self { data_store, .. } = self;
        Dependencies {
            url_frontier,
            data_store,
        }
    }

    pub fn data_store(
        self,
        data_store: Arc<RwLock<dyn DataStore<T, U> + Send + Sync>>,
    ) -> Dependencies<T, U> {
        let Self { url_frontier, .. } = self;
        Dependencies {
            url_frontier,
            data_store,
        }
    }

    pub fn build(self) -> Arc<Dependencies<T, U>> {
        Arc::new(Self {
            url_frontier: self.url_frontier,
            data_store: self.data_store,
        })
    }
}

// #[derive(Default)]
pub struct Dependencies<T: Clone + Hash + Eq, U> {
    pub url_frontier: Arc<RwLock<dyn Queue<T> + Send + Sync>>,
    pub data_store: Arc<RwLock<dyn DataStore<T, U> + Send + Sync>>,
}
