use actix_web::{test, web, App, HttpResponse};
use awc::Client;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::future::join_all;

#[inline]
pub fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

pub fn fibbo_bench(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

const STR: &str = "Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World \
                   Hello World Hello World Hello World Hello World Hello World";

// benchmark sending all requests at the same time
fn bench_async_burst(c: &mut Criterion) {
    let srv = test::start(|| {
        App::new().service(web::resource("/").route(web::to(|| HttpResponse::Ok().body(STR))))
    });

    // We are using System here, since Runtime requires preinitialized tokio
    // Maybe add to actix_rt docs
    let url = srv.url("/");
    let mut rt = actix_rt::System::new("test");

    c.bench_function("get_body_async_burst", move |b| {
        b.iter_custom(|iters| {
            let client = Client::new().get(url.clone()).freeze().unwrap();

            let start = std::time::Instant::now();
            // benchmark body
            let resps = rt.block_on(async move {
                let burst = (0..iters).map(|_| client.send());
                join_all(burst).await
            });
            let elapsed = start.elapsed();

            // if there are failed requests that might be an issue
            let failed = resps.iter().filter(|r| r.is_err()).count();
            if failed > 0 {
                eprintln!("failed {} requests (might be bench timeout)", failed);
            };

            elapsed
        })
    });
}

criterion_group!(server_benches, bench_async_burst, fibbo_bench);
criterion_main!(server_benches);
