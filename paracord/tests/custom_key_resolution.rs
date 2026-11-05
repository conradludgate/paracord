#[test]
fn allocated_custom_keys_still_resolve() {
    paracord::custom_key!(
        struct ValidKey;
    );
    let key = ValidKey::new("value");
    assert_eq!(key.as_str(), "value");
    assert_eq!(key.to_string(), "value");
    assert_eq!(ValidKey::try_new_existing("value"), Some(key));
    assert_eq!(ValidKey::iter().collect::<Vec<_>>(), [(key, "value")]);
}

#[test]
fn custom_key_layout_stays_compact() {
    paracord::custom_key!(
        struct LocalKey;
    );
    assert_eq!(
        std::mem::size_of::<LocalKey>(),
        std::mem::size_of::<paracord::Key>()
    );
    assert_eq!(
        std::mem::size_of::<Option<LocalKey>>(),
        std::mem::size_of::<paracord::Key>()
    );
    assert_eq!(
        std::mem::align_of::<LocalKey>(),
        std::mem::align_of::<paracord::Key>()
    );
}
