use criterion::{black_box, criterion_group, criterion_main, Criterion};
//use raft_kv_memstore::proto::{Command, Response};
use std::net::TcpStream;
use std::io::{Read, Write};
use raftkv::proto::Command;

fn write()  {
    let mut stream = TcpStream::connect("127.0.0.1:21001").unwrap();
    // for i in 1..1000{
    //     let key= format!("key{}",i);
    //     let value= format!("value{}",i);
    //     let msg=Command::Write {
    //         key: key,
    //         value: value,
    //     };
    //     //let req=serde_json::to_string(&msg).unwrap();
    //     let req=serde_json::to_string(&msg).unwrap();
    //     let res=stream.write(&req.into_bytes()).unwrap();
    //     let mut buf = vec![0u8; 4096];
    //     let n = stream.read(&mut buf).unwrap();
    //
    // }
    let msg=Command::Write {
        key: "key".to_string(),
        value: "value".to_string(),
    };
    let req=serde_json::to_string(&msg).unwrap();
    let res=stream.write(&req.into_bytes()).unwrap();
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).unwrap();
   //println!("write res{:?}",String::from_utf8(buf[..n].to_owned()).unwrap());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("write ", |b| b.iter(|| write()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

//  cargo run --bin  kv  --package raftkv  -- --id 1 --http-addr  "127.0.0.1:21001" --dir "./rkvstore/s1"
//  cargo run --bin  kv  --package raftkv  -- --id 2 --http-addr  "127.0.0.1:21002"  --dir "./rkvstore/s2"
//  cargo run --bin  kv  --package raftkv  -- --id 3 --http-addr  "127.0.0.1:21003"  --dir "./rkvstore/s3"

//cargo run --bin  tc  --package raftkv