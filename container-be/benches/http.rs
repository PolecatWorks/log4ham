use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use log4ham::{tokio_tools::rt_multithreaded, webserver::users::User};
use reqwest::Body;
use url::Url;

/// Benchmarking function for concurrent requests
/// This function benchmarks the performance of concurrent HTTP requests
/// using the `reqwest` library.


fn bench_http(c: &mut Criterion) {
    // Define the http url for the benchmarks as a Url

    let mut group = c.benchmark_group("http");

    let client_rt = rt_multithreaded(0).unwrap();
    let client = reqwest::Client::new();

    // http get to hello
    let url = Url::parse("http://localhost:8080/log4ham/hello").unwrap();

    group.bench_function("get hello", |b| {
        b.to_async(&client_rt).iter(|| async {
            let response = client.get(url.clone()).send().await;
            assert!(response.is_ok());
        })
    });

    // http post to user create
    let url = Url::parse("http://localhost:8080/log4ham/users").unwrap();

    let user = User::new("Ben", "Greene", "pw0");
    let user_json = serde_json::to_string(&user).unwrap();

    group.bench_function("post user", |b| {
        b.to_async(&client_rt).iter(|| async {
            let response = client.post(url.clone())
                .body(user_json.clone())
                .send().await;

            assert!(response.is_ok(),
                "Response: {:?}", response);
        })
    });

    drop(client_rt);

    let thread_tests = vec!(0,1,2,4,8);

    for threads in thread_tests {
        let bench_id = BenchmarkId::new("post user threads", threads);

        let client_rt = rt_multithreaded(threads).unwrap();

        group.bench_with_input(bench_id, &threads, |b, &num_threads| {
            b.to_async(&client_rt).iter(|| async {
                let response = client.post(url.clone())
                .body(user_json.clone())
                .send().await;

            assert!(response.is_ok(),
                "Response: {:?}", response);
            })
        });
    }




    group.finish();
}

criterion_group!(benches, bench_http);
criterion_main!(benches);
