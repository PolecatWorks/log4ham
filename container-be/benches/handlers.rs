use std::{env, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion};
use log4ham::{
    persistence::{
        DbConfig, ObjectStoreConfig, ObjectStoreType, PersistenceConfig, PersistenceState,
    },
    webserver::{
        lists::{self, List},
        ListOptions,
    },
    UrlWithUsernamePassword,
};
use url::Url;
use utils::rt_multithreaded;
mod utils;

pub fn bench_list_handlers(c: &mut Criterion) {
    let num_threads_client = 5;

    let client_rt = rt_multithreaded(num_threads_client);

    let db_config = PersistenceConfig {
        db: DbConfig {
            pool_size: 5,
            connection: UrlWithUsernamePassword {
                url: Url::parse(
                    &env::var("DATABASE_URL").expect("DATABASE_URL is defined with posgresl URL"),
                )
                .expect("Decoded URL from DATABASE_URL"),
                username: None,
                password: None,
            },
        },

    };

    //     pool_size: 5,
    //     connection: Url::parse(
    //         &env::var("DATABASE_URL").expect("DATABASE_URL is defined with posgresl URL"),
    //     )
    //     .expect("Decoded URL from DATABASE_URL"),
    // };

    let db_state = client_rt.block_on(async { PersistenceState::new(db_config).await.unwrap() });

    let options = ListOptions {
        offset: None,
        limit: None,
    };

    let mut group = c.benchmark_group("Handlers");

    {
        let body = List {
            id: None,
            name: "created record".to_owned(),
            active: None,
        };

        group.bench_function("create list", |b| {
            b.to_async(&client_rt)
                .iter(|| lists::handlers::create(body.clone(), db_state.pool_pg.clone()))
        });
    }

    group.bench_function("list list", |b| {
        b.to_async(&client_rt)
            .iter(|| lists::handlers::list(options.clone(), db_state.pool_pg.clone()))
    });

    {
        let id = 3;

        group.bench_function("read list", |b| {
            b.to_async(&client_rt)
                .iter(|| lists::handlers::read(id, db_state.pool_pg.clone()))
        });
    }

    {
        let id = 3;
        let body = List {
            id: Some(id),
            name: "created record".to_owned(),
            active: None,
        };

        group.bench_function("update list", |b| {
            b.to_async(&client_rt)
                .iter(|| lists::handlers::update(id, body.clone(), db_state.pool_pg.clone()))
        });
    }
    {
        let id = 2;

        group.bench_function("delete list", |b| {
            b.to_async(&client_rt)
                .iter(|| lists::handlers::delete(id, db_state.pool_pg.clone()))
        });
    }

    group.finish();

    client_rt.shutdown_timeout(Duration::from_secs(3));
}

criterion_group!(benches, bench_list_handlers);
criterion_main!(benches);
