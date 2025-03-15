use divan::{black_box_drop, Bencher};
use ustr::Ustr;

fn main() {
    divan::Divan::from_args()
        .sample_count(1000)
        .sample_size(1000)
        .threads([1, 2, 0])
        .run_benches();
}

#[divan::bench]
fn get_or_intern(b: Bencher) {
    b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
        .bench_refs(|s| black_box_drop(Ustr::from(s)));
}

#[divan::bench]
fn get(b: Bencher) {
    for x in 100000..=999999 {
        Ustr::from(&x.to_string());
    }

    b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
        .bench_refs(|s| black_box_drop(Ustr::from_existing(s)));
}

#[divan::bench]
fn resolve(b: Bencher) {
    let mut keys = vec![];
    for x in 100000..=999999 {
        keys.push(Ustr::from(&x.to_string()));
    }

    b.with_inputs(|| *fastrand::choice(&keys).unwrap())
        .bench_values(|key| black_box_drop(&*key));
}
