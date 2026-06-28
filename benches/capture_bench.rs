use audio_capture_rs_talaroana::{create_backend, CaptureConfig, SampleFormat};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn capture_init_bench(c: &mut Criterion) {
    c.bench_function("capture_config_default", |b| {
        b.iter(|| {
            let config = CaptureConfig::default();
            black_box(config);
        })
    });

    c.bench_function("create_backend", |b| {
        b.iter(|| {
            let backend = create_backend();
            black_box(backend);
        })
    });
}

criterion_group!(benches, capture_init_bench);
criterion_main!(benches);
