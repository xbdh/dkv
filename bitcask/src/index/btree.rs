use std::collections::BTreeMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::data::RecordPos;
use crate::index::Indexer;

pub struct Btree{
    pub tree: Arc<RwLock<BTreeMap<Vec<u8>, RecordPos>>>,
}

impl Indexer for Btree{
    fn Put(&self, key: Vec<u8>, pos: RecordPos) -> bool {
        let mut tree =self.tree.write();
        if tree.insert(key, pos).is_none(){
            return true
        }
        false
    }

    fn Get(&self, key: Vec<u8>) -> Option<RecordPos> {
        let tree = self.tree.read();
        tree.get(&key).cloned()
    }

    fn Delete(&self, key: Vec<u8>) -> bool {
        let mut tree = self.tree.write();
        if tree.remove(&key).is_some(){
            return true
        }
        false
    }
}

impl Btree{
    pub fn new() -> Self{
        Btree{
            tree: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}