pub mod btree;
use crate::data::RecordPos;


pub trait Indexer: Send + Sync{
    fn Put(&self, key: Vec<u8>, pos: RecordPos) -> bool;
    fn Get(&self, key: Vec<u8>) ->Option<RecordPos>;
    fn Delete(&self, key: Vec<u8>) -> bool;
}

pub fn NewIndexer() -> Box<dyn Indexer>{
    Box::new(btree::Btree::new())
}