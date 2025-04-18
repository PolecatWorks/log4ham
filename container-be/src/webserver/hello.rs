use log::info;
use warp::Filter;

use crate::MyState;

use super::with_state;

pub fn list(
    state: &MyState,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(with_state(state.clone()))
        .map(|state: MyState| {
            state.hello_counter.inc();
            info!("Hello: {}", state.hello_counter.get());
            warp::reply::json(&"Hello, world!")
        })
}

pub fn hello(
    state: &MyState,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list(state)
}

#[cfg(test)]
mod tests {
    use crate::config::MyConfig;

    use super::*;
    use warp::test::request;

    #[tokio::test]
    async fn test_hello() {
        let config = MyConfig::default();
        let state = MyState::new(&config).await.unwrap();
        let response = request()
            .method("GET")
            .path("/")
            .reply(&hello(&state))
            .await;

        assert_eq!(response.status(), 200);

        let body: String = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(
            body, "Hello, world!",
            "Response body should match: {}",
            body
        );
    }
}
