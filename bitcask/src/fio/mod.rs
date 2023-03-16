pub mod file_io;

use file_io::FileIO;
use std::path::PathBuf;
use crate::error::Result;


pub trait Store: Send + Sync{
    fn Read(&self,buf :&mut [u8],offset:u64) -> Result<usize>;
    fn Write(&self,buf :&[u8]) -> Result<usize>;
    fn Sync(&self) -> Result<()>;
}

pub fn NewStore(file_name:PathBuf) -> Result<Box<dyn Store>>{
    Ok(Box::new(FileIO::new(file_name)?))
}