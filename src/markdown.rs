use regex::Regex;

/** Stupid Markdown parser for parsing the void-packages manual.
**/
pub struct MarkdownParser {
    text: String,
}
impl MarkdownParser {
    pub fn new(text: &str) -> Self {
        // remove all links, they make parsing more complicated
        let replace_links_re = Regex::new(r#"<a id=".*?"><\/a>"#).unwrap();
        let text = replace_links_re.replace_all(text, "").into_owned();

        Self { text }
    }

    pub fn list_bullet_points(&self, section_title: &str) -> Option<Vec<&str>> {
        let text = &self.text;

        let section_start = text.find(section_title)?;
        let text = &text[section_start..];

        let start = text.find("\n-")?;
        let text = &text[start + 2..];

        let end = text.find("\n#").unwrap_or(text.len());
        let mut text = &text[..end];

        let mut bullet_points: Vec<&str> = vec![];
        while !text.is_empty() {
            match text.find("\n-") {
                Some(next_pos) => {
                    bullet_points.push(text[0..next_pos].trim());
                    text = &text[next_pos + 2..];
                }
                None => {
                    bullet_points.push(text.trim());
                    text = "";
                }
            }
        }

        Some(bullet_points)
    }
}

#[cfg(test)]
mod tests {
    use crate::markdown::MarkdownParser;

    #[test]
    fn parse_bullet_points() {
        let parser = MarkdownParser::new(
            r#"
## First section
- `a`
content of a

- `b`: content of b

- `c`

# New section
- `d`: content of d
            "#,
        );

        let bullet_points = parser.list_bullet_points("## First section");
        dbg!(&bullet_points);
        assert!(bullet_points.is_some_and(|p| p.len() == 3));
    }
}
