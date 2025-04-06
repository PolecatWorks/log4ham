use core::fmt;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::future::join_all;
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

    // let thread_tests = vec!(0,1,2,4,8);

    // for &threads in &thread_tests {
    //     let bench_id = BenchmarkId::new("post user threads", threads);

    //     let client_rt = rt_multithreaded(threads).unwrap();

    //     group.throughput(Throughput::Elements(1));

    //     group.bench_with_input(bench_id, &threads, |b, &num_threads| {
    //         b.to_async(&client_rt).iter(|| async {
    //             let response = client.post(url.clone())
    //             .body(user_json.clone())
    //             .send().await;

    //         assert!(response.is_ok(),
    //             "Response: {:?}", response);
    //         })
    //     });
    // }

    // for num_messages in [1,2,4,10,20,30,40,100] {
    for num_messages in [1,10,100] {

        for num_threads in [0,1,2,4,8] {
            let id = format!("{}-{}", num_messages, num_threads);
            let bench_id = BenchmarkId::new("post user", &id);

            let client_rt = rt_multithreaded(num_threads).unwrap();

            group.throughput(Throughput::Elements(num_messages));

            group.bench_with_input(bench_id, &id, |b, _id| {
                b.to_async(&client_rt).iter(|| async {

                    join_all( (0..num_messages)
                        .map(|i| async {
                            let response = client.post(url.clone())
                                .body(user_json.clone())
                                .send()
                                .await;
                            assert!(response.is_ok());
                        })).await;

                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_http);
criterion_main!(benches);
