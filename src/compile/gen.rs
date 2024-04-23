use crate::constants::RES_TEXT_RANGE_REG;
use regex::Regex;

#[derive(Debug, Default)]
pub struct CodeBlock {
    pub file_name: String,
    pub language: String,
    pub content: String,
}

impl CodeBlock {
    pub fn find_code_block(file_name: String, input: &str) -> Option<Self> {
        let re = Regex::new(RES_TEXT_RANGE_REG).unwrap();
        let captures = re.captures(input)?;

        Some(Self {
            file_name,
            language: captures["language"].to_owned(),
            content: captures["content"].to_owned(),
        })
    }
}
