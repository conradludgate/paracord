fn main() {
    divan::Divan::from_args()
        .sample_count(1000)
        .sample_size(1000)
        .threads([1, 2, 0])
        .main();
}

mod ustr {
    use divan::{black_box_drop, Bencher};
    use ustr::Ustr;

    #[divan::bench]
    fn get_or_intern(b: Bencher) {
        b.with_inputs(|| fastrand::u32(10000000..=99999999).to_string())
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
}

mod paracord {
    use divan::{black_box_drop, Bencher};
    use paracord::custom_key;

    custom_key!(struct Global);

    #[divan::bench]
    fn get_or_intern(b: Bencher) {
        b.with_inputs(|| fastrand::u32(10000000..=99999999).to_string())
            .bench_refs(|s| black_box_drop(Global::from_str_or_intern(s)));
    }

    #[divan::bench]
    fn get(b: Bencher) {
        for x in 100000..=999999 {
            Global::from_str_or_intern(&x.to_string());
        }

        b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
            .bench_refs(|s| black_box_drop(Global::try_from_str(s).unwrap()));
    }

    #[divan::bench]
    fn resolve(b: Bencher) {
        let mut keys = vec![];
        for x in 100000..=999999 {
            keys.push(Global::from_str_or_intern(&x.to_string()));
        }

        b.with_inputs(|| *fastrand::choice(&keys).unwrap())
            .bench_values(|key| black_box_drop(key.as_str()));
    }
}

mod lasso {
    use divan::{black_box_drop, Bencher};
    use foldhash::fast::RandomState;
    use lasso::{Spur, ThreadedRodeo};

    #[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
    #[repr(transparent)]
    struct Global(lasso::Spur);

    impl Global {
        fn rodeo() -> &'static ThreadedRodeo<Spur, RandomState> {
            static S: std::sync::OnceLock<ThreadedRodeo<Spur, RandomState>> =
                std::sync::OnceLock::new();
            S.get_or_init(
                || ThreadedRodeo::<Spur, RandomState>::with_hasher(RandomState::default()),
            )
        }

        pub fn get(s: &str) -> Option<Self> {
            Self::rodeo().get(s).map(Self)
        }

        pub fn get_or_intern(s: &str) -> Self {
            Self(Self::rodeo().get_or_intern(s))
        }

        pub fn resolve(self) -> &'static str {
            Self::rodeo().resolve(&self.0)
        }
    }

    #[divan::bench]
    fn get_or_intern(b: Bencher) {
        b.with_inputs(|| fastrand::u32(10000000..=99999999).to_string())
            .bench_refs(|s| black_box_drop(Global::get_or_intern(s)));
    }

    #[divan::bench]
    fn get(b: Bencher) {
        for x in 100000..=999999 {
            Global::get_or_intern(&x.to_string());
        }

        b.with_inputs(|| fastrand::u32(100000..=999999).to_string())
            .bench_refs(|s| black_box_drop(Global::get(s).unwrap()));
    }

    #[divan::bench]
    fn resolve(b: Bencher) {
        let mut keys = vec![];
        for x in 100000..=999999 {
            keys.push(Global::get_or_intern(&x.to_string()));
        }

        b.with_inputs(|| *fastrand::choice(&keys).unwrap())
            .bench_values(|key| black_box_drop(key.resolve()));
    }
}
