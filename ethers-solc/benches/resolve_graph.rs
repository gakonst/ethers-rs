//! read many sources
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use ethers_solc::Graph;
use ethers_solc::project_util::TempProject;

fn resolve_graph_bench_benchmark(c: &mut Criterion) {
    let tmp = TempProject::checkout("transmissions11/solmate").expect("failed to checkout project");
    let paths = tmp.paths();
    let mut group = c.benchmark_group("resolve graph");
    group.bench_function("sequential", |b| {
        b.iter(|| {
            Graph::resolve_sync(paths).unwrap();
        });
    });
    group.bench_function("parallel", |b| {
        b.iter(|| {
            Graph::resolve(paths).unwrap();
        });
    });
}


criterion_group!(benches, resolve_graph_bench_benchmark);
criterion_main!(benches);
