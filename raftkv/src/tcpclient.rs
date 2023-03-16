use tokio::time::{sleep, Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing_subscriber::fmt::time;
use raftkv::ExampleTypeConfig;
use raftkv::proto::{Command, Response};
use raftkv::proto::Command::Write;
use raftkv::typ::{ClientWriteResponse, RaftError};

#[tokio::main]
async fn main()->Result<(),Box<dyn std::error::Error>>{
    let mut listener = TcpStream::connect("127.0.0.1:21001").await?;
    init(&mut listener).await?;
    sleep(Duration::from_secs(2)).await;
    addlearner(&mut listener, 2, "127.0.0.1:21002").await?;
    addlearner(&mut listener, 3, "127.0.0.1:21003").await?;
    sleep(Duration::from_secs(2)).await;
    change_member(&mut listener, vec![1,2,3]).await?;
    sleep(Duration::from_secs(2)).await;

    // for i in 0..10 {
    //     let key = format!("key{}", i);
    //     let value = format!("value{}", i);
    //     write(&mut listener, &key ,&value).await?;
    // }
    //write(&mut listener, "key2", "value2").await?;
    //
    // sleep(Duration::from_secs(2)).await;
    // read(&mut listener, "key2").await?;
    // sleep(Duration::from_secs(2)).await;
    // metrics(&mut listener).await?;
    //
    //
    //
    // sleep(Duration::from_secs(3)).await;
    //
    // //read from foller
    // println!("read from follower");
    // let mut listener2 = TcpStream::connect("127.0.0.1:21002").await?;
    // let mut listener3 = TcpStream::connect("127.0.0.1:21003").await?;
    // read(&mut listener2, "key2").await?;
    // read(&mut listener3, "key2").await?;
    Ok(())
}

async fn init(steam: & mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let msg = Command::Init;
    let req = serde_json::to_string(&msg).unwrap();
    let res = steam.write_all(&req.into_bytes()).await?;
    let mut buf = vec![0u8; 4096];
    let n = steam.read(&mut buf).await?;
    let res: Response = serde_json::from_slice(&buf[..n]).unwrap();
    println!("init res: {:?}", res);
    Ok(())
}

async fn addlearner(steam: & mut TcpStream,id :u64,addr:&str) -> Result<(), Box<dyn std::error::Error>> {
    let msg = Command::AddLeader {
        node_id: id,
        addr: addr.to_string(),
    };
    let req=serde_json::to_string(&msg).unwrap();
    let res=steam.write_all(&req.into_bytes()).await?;
    let mut buf = vec![0u8; 4096];
    let n =steam.read(&mut buf).await?;
    //let res:Result<ClientWriteResponse, RaftError>=serde_json::from_slice(&buf[..n]).unwrap();
    println!("add learner res{:?}",String::from_utf8(buf[..n].to_owned()).unwrap());

    Ok(())
}
async fn change_member(steam: & mut TcpStream, members:Vec<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let msg=Command::ChangeMembership {
        members: members,
    };
    let req=serde_json::to_string(&msg).unwrap();
    let res=steam.write_all(&req.into_bytes()).await?;
    let mut buf = vec![0u8; 4096];
    let n = steam.read(&mut buf).await?;
    println!("change member res{:?}",String::from_utf8(buf[..n].to_owned()).unwrap());
    Ok(())
}
async fn write(steam: & mut TcpStream, key:&str,value:&str) -> Result<(), Box<dyn std::error::Error>> {
    let msg=Command::Write {
        key: key.to_string(),
        value: value.to_string(),
    };
    let req=serde_json::to_string(&msg).unwrap();
    let res=steam.write_all(&req.into_bytes()).await?;
    let mut buf = vec![0u8; 4096];
    let n = steam.read(&mut buf).await?;
    println!("write res{:?}",String::from_utf8(buf[..n].to_owned()).unwrap());
    Ok(())
}
async fn read(steam: & mut TcpStream, key:&str) -> Result<(), Box<dyn std::error::Error>> {
    let msg = Command::Read {
        key: key.to_string(),
    };
    let req=serde_json::to_string(&msg).unwrap();
    let res=steam.write_all(&req.into_bytes()).await?;
    let mut buf = vec![0u8; 4096];
    let n = steam.read(&mut buf).await?;
    println!("read res{:?}",String::from_utf8(buf[..n].to_owned()).unwrap());
    Ok(())
}
async   fn metrics(steam: & mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let msg = Command::Metrics;
    let req=serde_json::to_string(&msg).unwrap();
    let res=steam.write_all(&req.into_bytes()).await?;
    let mut buf = vec![0u8; 4096];
    let n = steam.read(&mut buf).await?;
    println!("metrics res{:?}",String::from_utf8(buf[..n].to_owned()).unwrap());
    Ok(())
}