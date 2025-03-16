fn main() {
    divan::Divan::from_args()
        .sample_count(1000)
        .sample_size(1000)
        .threads([1, 2, 0])
        .main();
}

mod paracord {
    use divan::{black_box_drop, Bencher};
    use paracord::ParaCord;

    #[divan::bench]
    fn get_or_intern(b: Bencher) {
        b.with_singleton(ParaCord::default)
            .with_inputs(|_: &ParaCord| fastrand::u32(100000..=999999).to_string())
            .bench_refs(|p, s| black_box_drop(p.get_or_intern(s)));
    }

    #[divan::bench]
    fn get(b: Bencher) {
        let p = ParaCord::default();
        for x in 100000..=999999 {
            p.get_or_intern(&x.to_string());
        }

        b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
            .bench_refs(|s| black_box_drop(p.get(s).unwrap()));
    }

    #[divan::bench]
    fn resolve(b: Bencher) {
        let p = ParaCord::default();
        let mut keys = vec![];
        for x in 100000..=999999 {
            keys.push(p.get_or_intern(&x.to_string()));
        }

        b.with_inputs(|| *fastrand::choice(&keys).unwrap())
            .bench_values(|key| black_box_drop(p.resolve(key)));
    }
}

mod lasso {
    use divan::{black_box_drop, Bencher};
    use foldhash::fast::RandomState;
    use lasso::{Spur, ThreadedRodeo};

    #[divan::bench]
    fn get_or_intern(b: Bencher) {
        b.with_singleton(
            || ThreadedRodeo::<Spur, RandomState>::with_hasher(RandomState::default()),
        )
        .with_inputs(|_: &ThreadedRodeo<_, _>| fastrand::u32(100000..=999999).to_string())
        .bench_refs(|p, s| black_box_drop(p.get_or_intern(s)));
    }

    #[divan::bench]
    fn get(b: Bencher) {
        let p = ThreadedRodeo::<Spur, RandomState>::with_hasher(RandomState::default());
        for x in 100000..=999999 {
            p.get_or_intern(x.to_string());
        }

        b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
            .bench_refs(|s| black_box_drop(p.get(s).unwrap()));
    }

    #[divan::bench]
    fn resolve(b: Bencher) {
        let p = ThreadedRodeo::<Spur, RandomState>::with_hasher(RandomState::default());
        let mut keys = vec![];
        for x in 100000..=999999 {
            keys.push(p.get_or_intern(x.to_string()));
        }

        b.with_inputs(|| *fastrand::choice(&keys).unwrap())
            .bench_values(|key| black_box_drop(p.resolve(&key)));
    }
}
