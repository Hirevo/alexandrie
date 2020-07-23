//! POLYVAL benchmark (using criterion)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use criterion_cycles_per_byte::CyclesPerByte;
use polyval::{universal_hash::UniversalHash, Polyval};

const KB: usize = 1024;

fn bench(c: &mut Criterion<CyclesPerByte>) {
    let mut group = c.benchmark_group("polyval");

    for size in &[KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB] {
        let buf = vec![0u8; *size];

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_function(BenchmarkId::new("update_padded", size), |b| {
            let mut m = Polyval::new(&Default::default());
            b.iter(|| m.update_padded(&buf));
        });
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(CyclesPerByte);
    targets = bench
);

criterion_main!(benches);
