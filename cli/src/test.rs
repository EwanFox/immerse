use super::*;

#[test]
fn test_is_kanji() {
    assert!(is_kanji('千'));
    assert!(is_kanji('本'));
    assert!(is_kanji('桜'));
    assert_ne!(is_kanji('に'), true);
    assert_ne!(is_kanji('h'), true);
    assert_ne!(is_kanji('1'), true)
} 