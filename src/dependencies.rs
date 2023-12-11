use std::sync::Arc;
use tokio::sync::RwLock;

pub type Deps = Arc<RwLock<Dependencies>>;

#[derive(Default)]
struct URLFrontier {}

#[derive(Default)]
struct DataStore {}

impl Dependencies {
    pub fn new() -> Deps {
        Arc::new(RwLock::new(Dependencies::default()))
    }
}

#[derive(Default)]
pub struct Dependencies {
    url_frontier: URLFrontier,
    data_store: DataStore,
}
