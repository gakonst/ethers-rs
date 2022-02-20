//! read many sources
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use ethers_core::rand;
use ethers_solc::artifacts::Source;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

fn read_all_benchmark(c: &mut Criterion) {
    let root = tempfile::tempdir().unwrap();
    let inputs = prepare_contracts(root.path(), 8);

    let mut group = c.benchmark_group("read many");
    group.sample_size(10);
    group.bench_function("sequential", |b| {
        b.iter(|| {
            Source::read_all(&inputs).unwrap();
        });
    });
    group.bench_function("parallel", |b| {
        b.iter(|| {
            Source::par_read_all(&inputs).unwrap();
        });
    });
}

fn prepare_contracts(root: &Path, num: usize) -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(num);
    for _ in 0..num {
        let path = root.join(format!("file{}.sol", num));
        let f = File::create(&path).unwrap();
        let mut writer = BufWriter::new(f);

        let mut rng = rand::thread_rng();

        // let's assume a solidity file is between 2kb and 16kb
        let n: usize = rng.gen_range(2..17);
        let s: String = rng.sample_iter(&Alphanumeric).take(n * 1024).map(char::from).collect();
        writer.write_all(s.as_bytes()).unwrap();
        writer.flush().unwrap();
        files.push(path)
    }
    files
}

criterion_group!(benches, read_all_benchmark);
criterion_main!(benches);
