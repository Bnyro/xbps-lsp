use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

pub struct TextDocument {
    text: String,
}
impl TextDocument {
    pub fn new(path: &Path) -> io::Result<Self> {
        let mut text = String::new();
        File::open(path)?.read_to_string(&mut text)?;

        Ok(Self { text })
    }

    pub fn identifier_at_pos(&self, line: u32, character: u32) -> Option<(String, u32, u32)> {
        let line = self.text.lines().nth(line as usize)?;
        const DELIMS: [char; 11] = [' ', '=', '+', '\n', '\t', '{', '}', '$', '"', '\'', '#'];

        if DELIMS.contains(&line.chars().nth(character as usize).unwrap()) {
            return None;
        }

        let mut start_idx = character as usize;
        while start_idx != 0 && !DELIMS.contains(&line.chars().nth(start_idx - 1).unwrap()) {
            start_idx -= 1;
        }

        let mut end_idx = character as usize;
        while end_idx != line.len() - 1 && !DELIMS.contains(&line.chars().nth(end_idx + 1).unwrap())
        {
            end_idx += 1;
        }

        Some((
            line[start_idx..=end_idx].to_string(),
            start_idx as u32,
            end_idx as u32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::text_document::TextDocument;

    #[test]
    fn identifier_position() {
        let document = TextDocument {
            text: r#"build_style=rust
do_install() {
    
}"#
            .to_string(),
        };

        assert_eq!(
            document.identifier_at_pos(0, 0),
            Some(("build_style".to_string(), 0, 10))
        );
        assert_eq!(
            document.identifier_at_pos(0, 10),
            Some(("build_style".to_string(), 0, 10))
        );
        assert_eq!(
            document.identifier_at_pos(0, 12),
            Some(("rust".to_string(), 12, 15))
        );
        assert_eq!(
            document.identifier_at_pos(1, 5),
            Some(("do_install()".to_string(), 0, 11))
        );
        assert_eq!(document.identifier_at_pos(2, 0), None);
    }
}
