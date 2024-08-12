use super::*;

#[test]
fn test_is_kanji() {
    assert!(is_kanji('千'));
    assert!(is_kanji('本'));
    assert!(is_kanji('桜'));
    assert!(!is_kanji('に'));
    assert!(!is_kanji('h'));
    assert!(!is_kanji('1'))
} 