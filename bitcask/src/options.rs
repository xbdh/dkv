use std::path::PathBuf;

#[derive(Debug, Clone,Default)]
pub struct Options {
    pub dir_path: PathBuf,
    pub max_segment_size: u64,
    pub sync_write : bool,
    //pub sync: bool,
}