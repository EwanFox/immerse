//Kanji Knowledge Levels
pub enum Knowledge {
    None,
    Seen,
    Recognize,
    Familiar,
    Write,
    Master,
}
#[derive(Debug)]
pub struct KanjiEntry {
    pub kanji: String,
    pub level: u8,
}

pub fn is_kanji(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |
        '\u{3400}'..='\u{4DBF}' |
        '\u{20000}'..='\u{2A6DF}' |
        '\u{2A700}'..='\u{2B73F}' |
        '\u{2B740}'..='\u{2B81F}' |
        '\u{2B820}'..='\u{2CEAF}' |
        '\u{2CEB0}'..='\u{2EBEF}'
    )
}
pub fn recommended_level(interval: u16) -> u8 {
    if interval >= 100 {
        3
    } else if interval >= 60 {
        2
    } else if interval > 0 {
        1
    } else {
        0
    }
}
