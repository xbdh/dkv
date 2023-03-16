
#[derive(Debug,Clone,Default)]
pub struct Record{
   pub key:Vec<u8>,
   pub value:Vec<u8>,
}

impl Record{
   pub fn Bytes(&self) -> Vec<u8>{
        let mut bytes = Vec::new();
        // add length of key
        bytes.extend_from_slice(&self.key.len().to_be_bytes());
        // add length of value
        bytes.extend_from_slice(&self.value.len().to_be_bytes());
        bytes.extend_from_slice(&self.key);
        bytes.extend_from_slice(&self.value);
        bytes
    }
}

impl From<&[u8]> for Record{
    fn from(bytes: &[u8]) -> Self {
        // keylen 8 bytes
        // valuelen 8 bytes
        
        let keylen = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let valuelen = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        let key = bytes[16..16+keylen as usize].to_vec();
        let value = bytes[16+keylen as usize..16+keylen as usize+valuelen as usize].to_vec();
        Record{
            key,
            value,
        }
    }
}

#[derive(Debug,Clone)]
pub struct RecordPos{
     pub(crate) fid :u32,
     pub(crate) offset:u64,
}
#[derive(Debug,Clone,Default)]
pub struct ReadRecordRes {
    pub record:Record,
    pub size:u64,
}