use async_trait::async_trait;
use crossbeam_queue::SegQueue;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub trait Queue<T>: Enqueue<T> + Dequeue<T> {}

#[async_trait]
pub trait Dequeue<T> {
    async fn dequeue(&mut self) -> Option<T>;
}

pub trait Enqueue<T> {
    fn enqueue(&mut self, value: T);
}

#[derive(Default)]
pub struct URLFrontier<T> {
    queue: Arc<SegQueue<T>>,
    delay_s: Option<u64>,
}

impl<T> Queue<T> for URLFrontier<T> where T: Send {}

#[async_trait]
impl<T: Send> Dequeue<T> for URLFrontier<T> {
    async fn dequeue(&mut self) -> Option<T> {
        if let Some(delay_s) = self.delay_s {
            sleep(Duration::from_secs(delay_s)).await;
        }
        self.queue.pop()
    }
}

impl<T> Enqueue<T> for URLFrontier<T> {
    fn enqueue(&mut self, value: T) {
        self.queue.push(value)
    }
}

#[derive(Default)]
pub struct URLFrontierBuilder<T> {
    queue: SegQueue<T>,
    delay_s: Option<u64>,
}

impl<T: Default> URLFrontierBuilder<T> {
    pub fn new() -> URLFrontierBuilder<T> {
        URLFrontierBuilder {
            queue: SegQueue::new(),
            delay_s: None,
        }
    }

    pub fn value(self, value: T) -> URLFrontierBuilder<T> {
        self.queue.push(value);
        self
    }

    pub fn delay_s(self, delay_s: u64) -> URLFrontierBuilder<T> {
        if delay_s > 0 {
            let Self { queue, .. } = self;
            URLFrontierBuilder {
                delay_s: Some(delay_s),
                queue,
            }
        } else {
            self
        }
    }

    pub fn build(self) -> URLFrontier<T> {
        URLFrontier {
            queue: self.queue.into(),
            delay_s: self.delay_s,
        }
    }
}

#[cfg(test)]
mod url_frontier_tests {
    use super::Dequeue;
    use super::Enqueue;
    use super::URLFrontier;
    use super::URLFrontierBuilder;

    #[test]
    fn url_frontier_builder_builds_url_frontier() {
        let url_frontier = URLFrontierBuilder::new()
            .delay_s(1)
            .value("one".to_string())
            .build();

        assert!(url_frontier.delay_s == Some(1));
        assert!(url_frontier.queue.pop() == Some("one".to_owned()));
    }

    #[tokio::test]
    async fn url_frontier_dequeues_value() {
        let mut url_frontier = URLFrontierBuilder::new()
            .delay_s(0)
            .value("one".to_string())
            .build();

        let val = url_frontier.dequeue().await;

        assert_eq!(val, Some("one".to_owned()));
    }

    #[tokio::test]
    async fn url_frontier_dequeues_none_if_there_are_no_values_in_the_queue() {
        let mut url_frontier: URLFrontier<String> = URLFrontierBuilder::new().delay_s(0).build();

        let val = url_frontier.dequeue().await;

        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn url_frontier_enqueues_value() {
        let mut url_frontier = URLFrontierBuilder::new().delay_s(0).build();

        url_frontier.enqueue("two".to_owned());
        let val = url_frontier.dequeue().await;

        assert_eq!(val, Some("two".to_owned()));
        assert_eq!(url_frontier.dequeue().await, None);
    }
}
