use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use log4ham::app_schema::{schema_person_string, schema_string, validate, Person};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("schema generate Person", |b| b.iter(schema_person_string));

    c.bench_function("schema Generate Vec<Person>", |b| {
        b.iter(schema_string::<Vec<Person>>)
    });

    let error_limit = 2;
    let schema = schema_string::<Vec<Person>>().expect("got schema");

    let mut group = c.benchmark_group("validate");

    for size in [
        0, 1, 10, 100, 1000,
        10000,
        // 1,2,3,4,5,6,7,8,9,
        // 10,20,30,40,50,60,70,80,90,
        // 100,200,300,400,500,600,700,800,900,
        // 1000,2000,3000,4000,5000,6000,7000,8000,9000,
        // 10000,
        // 100000,
        // 1000000,
    ]
    .iter()
    {
        let my_data_vec: Vec<_> = (0..*size)
            .map(|x| Person {
                name: format!("name-{:08}", x),
                age: x,
            })
            .collect();
        let my_data_string = serde_json::to_string_pretty(&my_data_vec).expect("to string");
        let my_data_bytes = my_data_string.as_bytes();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| validate(&schema, my_data_bytes, error_limit).expect("validated"));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
