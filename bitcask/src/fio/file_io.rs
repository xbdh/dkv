use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::prelude::FileExt;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::error::{errors, Result};
use super::Store;

#[derive(Clone)]
pub struct  FileIO {
    fd:Arc<RwLock<File>>
}
impl Default for FileIO {
    fn default() -> Self {
        let file = File::open("/dev/null").unwrap();
        let fd = Arc::new(RwLock::new(file));
        Self { fd }
    }
}

impl Store for FileIO{
    fn Read(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let file=self.fd.read();
        match file.read_at(buf, offset){
            Ok(size) => Ok(size),
            Err(e) => Err(errors::FailedToReadFromFile),
        }

    }

    fn Write(&self, buf: &[u8]) -> Result<usize> {
        let mut file=self.fd.write();
        match file.write_all(buf) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(errors::FailedToWriteFromFile),
        }
    }

    fn Sync(&self) -> Result<()> {
        let mut file=self.fd.write();
        match file.sync_all() {
            Ok(_) => Ok(()),
            Err(e) => Err(errors::FailedTosyncFromFile),
        }
    }
}
impl FileIO{
    pub fn new(path: PathBuf) -> Result<Self>{
        match OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(path){
                Ok(file) => Ok(FileIO{
                    fd: Arc::new(RwLock::new(file)),
                }),
                Err(e) => Err(errors::FialedToOpenFile),
            }
    }
}