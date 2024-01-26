use crate::{
    data_store::{DataStore, Store},
    url_frontier::{Queue, URLFrontierBuilder},
};
use std::{fmt::Debug, hash::Hash, ops::Deref, sync::Arc};
use tokio::sync::RwLock;

pub type Deps<T, U> = Arc<RwLock<Dependencies<T, U>>>;
pub type DepsConcrete = Arc<Dependencies<String, String>>;

// Implement the Deref trait in order to access impl Queue without having to .0
impl<T> Deref for Frontier<T> {
    type Target = Arc<RwLock<dyn Queue<T> + Send + Sync>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Frontier<T>(pub Arc<RwLock<dyn Queue<T> + Send + Sync>>);

#[derive(Default)]
pub struct UrlFrontierOptions<T> {
    pub delay_s: Option<u64>,
    pub uri: T,
}

pub fn url_frontier<T: Send + Sync + Default + 'static>(
    opts: UrlFrontierOptions<T>,
) -> Frontier<T> {
    let url_frontier = URLFrontierBuilder::default()
        .delay_s(opts.delay_s.unwrap_or(0))
        .value(opts.uri)
        .build();

    Frontier(Arc::new(RwLock::new(url_frontier)))
}

// Implement the Deref trait in order to access impl Queue without having to .0
impl<T, U> Deref for MemoryStore<T, U> {
    type Target = Arc<RwLock<dyn DataStore<T, U> + Send + Sync>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MemoryStore<T, U>(pub Arc<RwLock<dyn DataStore<T, U> + Send + Sync>>);

pub fn data_store<
    T: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    U: Send + Sync + Debug + 'static,
>() -> MemoryStore<T, U> {
    let store = Store::new();
    MemoryStore(Arc::new(RwLock::new(store)))
}

impl<
        T: Clone + Default + Debug + Hash + Eq + Send + Sync + 'static,
        U: Send + Debug + Sync + Default + 'static,
    > Default for Dependencies<T, U>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        T: Clone + Default + Debug + Hash + Eq + Send + Sync + 'static,
        U: Send + Debug + Sync + Default + 'static,
    > Dependencies<T, U>
{
    pub fn new() -> Dependencies<T, U> {
        Dependencies {
            url_frontier: url_frontier(UrlFrontierOptions::default()),
            data_store: data_store(),
        }
    }

    pub fn url_frontier(self, url_frontier: Frontier<T>) -> Dependencies<T, U> {
        let Self { data_store, .. } = self;
        Dependencies {
            url_frontier,
            data_store,
        }
    }

    pub fn data_store(self, data_store: MemoryStore<T, U>) -> Dependencies<T, U> {
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

pub struct Dependencies<T: Clone + Hash + Eq, U> {
    pub url_frontier: Frontier<T>,
    pub data_store: MemoryStore<T, U>,
}
