use crate::{
    data_store::{DataStore, Store},
    url_frontier::{Queue, URLFrontier, URLFrontierBuilder},
};
use std::{default, fmt::Debug, hash::Hash, sync::Arc};
use tokio::sync::RwLock;

pub type Deps<T, U> = Arc<RwLock<Dependencies<T, U>>>;
pub type DepsConcrete = Arc<Dependencies<String, String>>;

pub struct Frontier<T>(pub Arc<RwLock<dyn Queue<T> + Send + Sync>>);

pub struct Options {
    pub delay_s: Option<u64>,
    pub uri: String,
}

pub fn q<T: Send + Sync + Default + 'static>(opts: Options) -> Frontier<T> {
    // let u = URLFrontierBuilder::new()
    //     .delay_s(opts.delay_s.unwrap())
    //     .value(opts.uri)
    //     .build();

    let mut u = URLFrontier::default();
    u.set_delay(opts.delay_s);

    Frontier(Arc::new(RwLock::new(u)))
}

// trait Depos {
//     type D: DataStore;
//     type F: Queue;

//     fn f(&self) -> &Self::F;
//     fn d(&self) -> &Self::D;
// }

impl<
        T: Clone + Default + Debug + Hash + Eq + Send + Sync + 'static,
        U: Send + Debug + Sync + Default + 'static,
    > Dependencies<T, U>
{
    pub fn new(opts: Options) -> Dependencies<T, U> {
        Dependencies {
            url_frontier: q(opts),
            data_store: Arc::new(RwLock::new(Store::default())),
        }
    }

    pub fn url_frontier(self, url_frontier: Frontier<T>) -> Dependencies<T, U> {
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

pub struct Dependencies<T: Clone + Hash + Eq, U> {
    pub url_frontier: Frontier<T>,
    pub data_store: Arc<RwLock<dyn DataStore<T, U> + Send + Sync>>,
}
