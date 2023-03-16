use std::fmt::Debug;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::data::{ReadRecordRes, Record};
use crate::fio;
use crate::error::Result;
use crate::fio::file_io::FileIO;
use log::{info,debug};

pub  struct Semgent {
    pub(crate) fileid:Arc<RwLock<u32>>,
    pub(crate) write_offset:Arc<RwLock<u64>>,
    file: Arc<Box<dyn fio::Store>>,
}
impl Default for Semgent{
    fn default() -> Self {
        let file = Box::new(FileIO::default());
        Self {
            fileid: Default::default(),
            write_offset: Default::default(),
            file: Arc::new(file),
        }
    }
}
impl Clone for Semgent{
    fn clone(&self) -> Self {
        Self {
            fileid: self.fileid.clone(),
            write_offset: self.write_offset.clone(),
            file: self.file.clone(),
        }
    }
}

impl Debug for Semgent{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Semgent")
            .field("fileid", &self.fileid)
            .field("write_offset", &self.write_offset)
            //.field("file", &self.file)
            .finish()
    }
}

impl Semgent{
    pub fn new(dir_path:PathBuf,fileid:u32) -> Result<Self>{
        let file_name=std::format!("{}",fileid)+ ".store";
        let full_path = dir_path.join(file_name);
        info!("new store file full_path:{}",full_path.to_str().unwrap());
        let fileio=fio::NewStore(full_path)?;

        Ok(Semgent{
            fileid: Arc::new(RwLock::new(fileid)),
            write_offset: Default::default(),
            file: Arc::new(fileio),
        })
    }
    pub fn Sync(&self) -> Result<()>{
        self.file.Sync()
    }
    pub fn ReadRecord(&self,offset:u64) -> Result<ReadRecordRes>{
        // 8位key的长度 8位value的长度，接着是key和value
        let mut buf = [0u8;16];
        let size = self.file.Read(&mut buf,offset)?;
        if size != 16{
            return Err(crate::error::errors::FailedToReadFromFile);
        }
        let key_size = u64::from_be_bytes(buf[0..8].try_into().unwrap());
        let value_size = u64::from_be_bytes(buf[8..16].try_into().unwrap());
        let mut key_buf = vec![0u8;key_size as usize];
        let mut value_buf = vec![0u8;value_size as usize];
        let key_size = self.file.Read(&mut key_buf,offset+16)?;
        let value_size = self.file.Read(&mut value_buf,offset+16+key_size as u64)?;

        if key_size != key_size as usize || value_size != value_size as usize{
            return Err(crate::error::errors::FailedToReadFromFile);
        }
        let record = Record{
            key: key_buf,
            value: value_buf,
        };
        let read_record_res = ReadRecordRes{
            record: record,
            size:   16 + key_size as u64 + value_size as u64,
        };

        Ok(read_record_res)
    }
    pub fn Append(&self,b:&[u8]) -> Result<u64>{
        info!("segment append bytelen {:?}",b.len());
        let mut write_offset = self.write_offset.write();
        let size = self.file.Write(b)?;
        //let recordwritesize = size;
        *write_offset += size as u64;
        Ok(size as u64)
    }
}