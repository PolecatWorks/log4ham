use warp::Filter;

pub fn list() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::json(&"Hello, world!"))
}

pub fn hello() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list()
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::test::request;

    #[tokio::test]
    async fn test_hello() {
        let response = request().method("GET").path("/").reply(&hello()).await;

        assert_eq!(response.status(), 200);

        let body: String = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(
            body, "Hello, world!",
            "Response body should match: {}",
            body
        );
    }
}
