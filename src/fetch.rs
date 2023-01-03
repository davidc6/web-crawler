use reqwest::{Client, Error, IntoUrl};

pub trait Fetch {
    fn new() -> Self;
    fn get<T: IntoUrl + Send>(
        &self,
        url: T,
    ) -> impl std::future::Future<Output = Result<String, Error>> + std::marker::Send;
}

#[derive(Default)]
pub struct HttpFetch {
    client: Client,
}

impl Fetch for HttpFetch {
    fn new() -> HttpFetch {
        HttpFetch {
            client: Client::new(),
        }
    }

    async fn get<T: IntoUrl + Send>(&self, url: T) -> Result<String, Error> {
        let response = self.client.get(url).send().await?;
        response.text().await
    }
}

#[cfg(test)]
mod fetch_tests {
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use crate::fetch::{Fetch, HttpFetch};

    #[tokio::test]
    async fn get_makes_a_call_and_returns_a_response() {
        let f: HttpFetch = Fetch::new();

        let mock_server = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let response = f.get(&mock_server.uri()).await;

        assert_eq!(response.unwrap(), "Hello".to_owned());
    }
}
