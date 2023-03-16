
fn main(){
   let log=bitcask::db::Engine::new(bitcask::options::Options {
        dir_path: "./store".to_string().parse().unwrap(),
        max_segment_size: 100,
        sync_write: true,
    }).unwrap();
    for i in 0..10{
        let key= format!("key{}",i);
        let value= format!("value{}",i);
        log.append(key,value).unwrap();
    }
    for i in 0..10{
        let key= format!("key{}",i);
        //let value= format!("value{}",i);
        let res=log.get(key).unwrap();
        println!("value:{:?}",res);
    }

}