//! read many sources
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use ethers_solc::{project_util::TempProject, Graph, ProjectPathsConfig};

fn resolve(id: &str, paths: &ProjectPathsConfig, c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("resolve {}", id));
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

fn resolve_solmate_benchmark(c: &mut Criterion) {
    let id = "transmissions11/solmate";
    let tmp = TempProject::checkout(id).expect("failed to checkout project");
    resolve(id, tmp.paths(), c)
}

fn resolve_spells_benchmark(c: &mut Criterion) {
    if let Ok(spells) = std::env::var("SPELLS_DIR") {
        // cloning spells repo takes forever so this is only run when path to local dir is set
        let id = "makerdao/spells-mainnet";
        let paths = ProjectPathsConfig::dapptools(spells).unwrap();
        resolve(id, &paths, c)
    }
}

criterion_group!(benches, resolve_spells_benchmark, resolve_solmate_benchmark);
criterion_main!(benches);
