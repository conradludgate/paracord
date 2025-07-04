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
        b.with_inputs(|| fastrand::u32(10000000..=99999999))
            .bench_values(|s| black_box_drop(Ustr::from(itoa::Buffer::new().format(s))));
    }

    #[divan::bench]
    fn get(b: Bencher) {
        for x in 1000000..=1999999 {
            Ustr::from(itoa::Buffer::new().format(x));
        }

        b.with_inputs(|| fastrand::u32(1000000..=1999999))
            .bench_values(|s| black_box_drop(Ustr::from_existing(itoa::Buffer::new().format(s))));
    }

    #[divan::bench]
    fn resolve(b: Bencher) {
        let mut keys = vec![];
        for x in 2000000..=2999999 {
            keys.push(Ustr::from(itoa::Buffer::new().format(x)));
        }

        b.with_inputs(|| *fastrand::choice(&keys).unwrap())
            .bench_values(|key| black_box_drop(&*key));
    }

    #[divan::bench]
    fn get_or_intern_existing(b: Bencher) {
        for x in 3000000..=3999999 {
            Ustr::from(itoa::Buffer::new().format(x));
        }

        b.with_inputs(|| fastrand::u32(3000000..=3999999))
            .bench_values(|s| black_box_drop(Ustr::from(itoa::Buffer::new().format(s))));
    }
}

mod paracord {
    use divan::{black_box_drop, Bencher};
    use paracord::custom_key;

    custom_key!(struct Global);

    #[divan::bench]
    fn get_or_intern(b: Bencher) {
        b.with_inputs(|| fastrand::u32(10000000..=99999999))
            .bench_values(|s| black_box_drop(Global::new(itoa::Buffer::new().format(s))));
    }

    #[divan::bench]
    fn get(b: Bencher) {
        for x in 1000000..=1999999 {
            Global::new(itoa::Buffer::new().format(x));
        }

        b.with_inputs(|| fastrand::u32(1000000..=1999999))
            .bench_values(|s| {
                black_box_drop(Global::try_new_existing(itoa::Buffer::new().format(s)).unwrap())
            });
    }

    #[divan::bench]
    fn resolve(b: Bencher) {
        let mut keys = vec![];
        for x in 2000000..=2999999 {
            keys.push(Global::new(itoa::Buffer::new().format(x)));
        }

        b.with_inputs(|| *fastrand::choice(&keys).unwrap())
            .bench_values(|key| black_box_drop(key.as_str()));
    }

    #[divan::bench]
    fn get_or_intern_existing(b: Bencher) {
        for x in 3000000..=3999999 {
            Global::new(itoa::Buffer::new().format(x));
        }

        b.with_inputs(|| fastrand::u32(3000000..=3999999))
            .bench_values(|s| black_box_drop(Global::new(itoa::Buffer::new().format(s))));
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
        b.with_inputs(|| fastrand::u32(10000000..=99999999))
            .bench_values(|s| black_box_drop(Global::get_or_intern(itoa::Buffer::new().format(s))));
    }

    #[divan::bench]
    fn get(b: Bencher) {
        for x in 1000000..=1999999 {
            Global::get_or_intern(itoa::Buffer::new().format(x));
        }

        b.with_inputs(|| fastrand::u32(1000000..=1999999))
            .bench_values(|s| black_box_drop(Global::get(itoa::Buffer::new().format(s)).unwrap()));
    }

    #[divan::bench]
    fn resolve(b: Bencher) {
        let mut keys = vec![];
        for x in 2000000..=2999999 {
            keys.push(Global::get_or_intern(itoa::Buffer::new().format(x)));
        }

        b.with_inputs(|| *fastrand::choice(&keys).unwrap())
            .bench_values(|key| black_box_drop(key.resolve()));
    }

    #[divan::bench]
    fn get_or_intern_existing(b: Bencher) {
        for x in 3000000..=3999999 {
            Global::get_or_intern(itoa::Buffer::new().format(x));
        }

        b.with_inputs(|| fastrand::u32(3000000..=3999999))
            .bench_values(|s| black_box_drop(Global::get_or_intern(itoa::Buffer::new().format(s))));
    }
}
