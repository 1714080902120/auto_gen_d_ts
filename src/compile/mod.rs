use crate::file_io::read::File;

pub mod gen;
pub mod parse;
pub mod transform;

#[derive(Debug, Default)]
pub struct Transformed {
    pub file: File,
    pub filter_code: Vec<Vec<String>>,
}
