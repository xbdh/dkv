use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs::read_dir;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::data::{ReadRecordRes, Record, RecordPos};
use crate::index::NewIndexer;
use crate::index::Indexer;
use crate::options::Options;
use crate::segment::{ Semgent};
use crate::error::Result;
use log::{info,debug};

pub struct Engine{
    ops:Arc<Options>,
    active_segment:Arc<RwLock<Semgent>>,
    old_segments:Arc<RwLock<HashMap<u32,Semgent>>>,
    index:Box<dyn Indexer>,
    fids:Vec<u32>
}
impl Clone for Engine{
    fn clone(&self) -> Self {
        Engine{
            ops:self.ops.clone(),
            active_segment:Arc::new(RwLock::new(self.active_segment.read().clone())),
            old_segments:Arc::new(RwLock::new(self.old_segments.read().clone())),
            index:NewIndexer(),
            fids:self.fids.clone(),
        }
    }
}
impl Engine {
    pub fn new(ops:Options) -> Result<Self>{
        info!("new engine");
        let idx=NewIndexer();
        let mut eg =Engine{
            ops:Arc::new(ops),
            active_segment:Default::default(),
            old_segments:Default::default(),
            index:idx,
            fids:Vec::new(),
        };

        eg.init_segment()?;

        eg.init_index()?;

        Ok(eg)


    }

    pub fn init_segment(&mut self) -> Result<()>{
        info!("init_segment");
        let mut fids=Vec::new();

        let res=read_dir(&self.ops.dir_path).unwrap();
        for entry in res{
            let entry=entry.unwrap();
            let file_name_os_str=entry.file_name();
            let file_name_str=file_name_os_str.to_str().unwrap();
            //info!("file_name_str:{}",file_name_str);
            let file_name=file_name_str.to_string();
            let id = file_name.trim_end_matches(".store");
            let file_id=id.parse::<u32>().unwrap();
            fids.push(file_id);
        }
        fids.sort();

        for i in 0..fids.len(){
            let fid=fids[i];
            self.new_segmnet(self.ops.dir_path.clone(), fid)?;
        }
        if fids.len() == 0{
            self.new_segmnet(self.ops.dir_path.clone(), 0)?;
        }

        self.fids=fids;
        // info!("after init segmet len of old {:?}",self.old_segments.read().len());
        // info!("after init segmet  active fid id {:?}",self.active_segment.read().fileid.read().clone());
        // info!("after init segmet  fids {:?}",self.fids);
        Ok(())
    }
    pub fn init_index(&self) -> Result<()>{
        info!("init_index");
        let mut old_segments=self.old_segments.write();
        let mut active_segment =self.active_segment.write();

        for (i,fid) in self.fids.iter().enumerate(){
            let mut offset=0;
            loop{
                let log_record_res = match *fid == active_segment.fileid.read().clone() {
                    true => active_segment.ReadRecord(offset),
                    false => {
                        let data_file = old_segments.get(fid).unwrap();
                        data_file.ReadRecord(offset)
                    }
                };
                let (log_record,log_record_size)=match log_record_res {
                    Ok(Read ) => (Read.record, Read.size),
                    Err(_) => {
                        break;
                    }
                };
                let recordpos=RecordPos{
                    fid:*fid,
                    offset:offset,
                };
                self.index.Put(log_record.key,recordpos);
                offset += log_record_size;
            }
            if i == self.fids.len() - 1 {
                *active_segment.write_offset.write() = offset;
            }
        }
        Ok(())
    }



    pub fn append(&self, key:String, value :String) -> Result<()>{
        info!("engine append");
        let record=Record{
            key:key.into_bytes(),
            value:value.into_bytes(),
        };
        let recordpos=self.append_record(&record)?;
        info!("recordpos:{:?}",recordpos);
        self.index.Put(record.key,recordpos);

        Ok(())
    }
    pub fn get(&self, key:String) -> Result<String>{
        info!("get");
        let recordpos=self.index.Get(key.into_bytes());
        if recordpos.is_none(){
            return Ok("".to_string());
        }
        let recordpos=recordpos.unwrap();
        let mut active_segment =self.active_segment.write();
        if recordpos.fid==active_segment.fileid.read().clone(){
            let res=active_segment.ReadRecord(recordpos.offset);
            if res.is_err(){
                return Ok("".to_string());
            }
            let res=res.unwrap();
            let sv=String::from_utf8(res.record.value).unwrap();
            return Ok(sv);
        }else {
            let mut old_segments=self.old_segments.write();
            let old_segment=old_segments.get(&recordpos.fid);
            if old_segment.is_none(){
                return Ok("".to_string());
            }
            let old_segment=old_segment.unwrap();
            let res=old_segment.ReadRecord(recordpos.offset);
            if res.is_err(){
                return Ok("".to_string());
            }
            let res=res.unwrap();
            let sv=String::from_utf8(res.record.value).unwrap();
            return Ok(sv);
        }
    }

    pub fn append_record(&self, record:&Record) -> Result<RecordPos>{
        info!("enginne append_record");
        let active_segment_init_fid=self.get_active_segment_fileid();
        let active_segment_init_write_offset=self.get_active_segment_write_offset();

        let bytes=record.Bytes();

        // 有可能新建段
        if active_segment_init_write_offset+bytes.len() as u64 > self.ops.max_segment_size{
            info!("need build a new_segmnet");
            self.new_segmnet(self.ops.dir_path.clone(),active_segment_init_fid+1)?;
        }
        info!("start append_record in active_segment");
        //let size=active_segment.Append(&bytes)?;
        let size=match self.active_segment.write().Append(&bytes){
            Ok(size) => size,
            Err(e) => {
                info!("append_record error:{:?}",e);
                return Err(e);
            }
        };
        let recordpos=RecordPos{
            fid:self.get_active_segment_fileid(),
            offset:self.get_active_segment_write_offset()-size, //segment 已经对write_offset 加了size，所以这里不用加
        };
        Ok(recordpos)
    }
    pub fn new_segmnet(&self, dir_path :PathBuf, fid:u32) -> Result<()>{
        let seg=Semgent::new(dir_path,fid)?;
        let mut old_segments=self.old_segments.write();
        let mut active_segment =self.active_segment.write();
        let active_fid=active_segment.fileid.read().clone();
        old_segments.insert(active_fid,active_segment.clone());
        *active_segment=seg.clone();
        Ok(())
    }
    fn get_active_segment_fileid(&self) -> u32{
        let active_segment =self.active_segment.read();
        let id=active_segment.fileid.read().clone();
        id
    }
    fn get_active_segment_write_offset(&self) -> u64{
        let active_segment =self.active_segment.read();
        let offset=active_segment.write_offset.read().clone();
        offset
    }
}

impl Debug for Engine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Engine {{  }}")
    }
}

impl Default for Engine{
    fn default() -> Self {
        Engine {
            active_segment:Arc::new(RwLock::new(Semgent::default())),
            old_segments:Arc::new(RwLock::new(HashMap::new())),
            index:NewIndexer(),
            fids:Vec::new(),
            ops: Arc::from(Options::default()),
        }
    }
}


