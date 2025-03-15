use divan::{black_box_drop, Bencher};
use paracord::custom_key;

fn main() {
    divan::Divan::from_args()
        .sample_count(1000)
        .sample_size(1000)
        .threads([1, 2, 0])
        .run_benches();
}

custom_key!(struct Global);

#[divan::bench]
fn get_or_intern(b: Bencher) {
    b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
        .bench_local_refs(|s| black_box_drop(Global::get_or_intern(s)));
}

#[divan::bench]
fn get(b: Bencher) {
    for x in 100000..=999999 {
        Global::get_or_intern(&x.to_string());
    }

    b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
        .bench_local_refs(|s| black_box_drop(Global::get(s).unwrap()));
}

#[divan::bench]
fn resolve(b: Bencher) {
    let mut keys = vec![];
    for x in 100000..=999999 {
        keys.push(Global::get_or_intern(&x.to_string()));
    }

    b.with_inputs(|| *fastrand::choice(&keys).unwrap())
        .bench_local_refs(|key| black_box_drop(key.resolve()));
}
