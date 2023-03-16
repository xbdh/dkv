#![allow(clippy::uninlined_format_args)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use actix_web::middleware;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use openraft::{BasicNode, MessageSummary};
use openraft::Config;
use openraft::Raft;
use serde::Serialize;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::app::ExampleApp;
use crate::network::api;
use crate::network::management;
use crate::network::raft;
use crate::network::raft_network_impl::ExampleNetwork;
use crate::store::ExampleRequest;
use crate::store::ExampleResponse;
use crate::store::ExampleStore;

pub mod app;
pub mod client;
pub mod network;
pub mod store;
pub mod proto;

use proto::{Command, Response};

pub type ExampleNodeId = u64;

openraft::declare_raft_types!(
    /// Declare the type configuration for example K/V store.
    pub ExampleTypeConfig: D = ExampleRequest, R = ExampleResponse, NodeId = ExampleNodeId, Node = BasicNode
);

pub type ExampleRaft = Raft<ExampleTypeConfig, ExampleNetwork, Arc<ExampleStore>>;

pub mod typ {
    use openraft::BasicNode;

    use crate::ExampleNodeId;
    use crate::ExampleTypeConfig;

    pub type RaftError<E = openraft::error::Infallible> = openraft::error::RaftError<ExampleNodeId, E>;
    pub type RPCError<E = openraft::error::Infallible> =
        openraft::error::RPCError<ExampleNodeId, BasicNode, RaftError<E>>;

    pub type ClientWriteError = openraft::error::ClientWriteError<ExampleNodeId, BasicNode>;
    pub type CheckIsLeaderError = openraft::error::CheckIsLeaderError<ExampleNodeId, BasicNode>;
    pub type ForwardToLeader = openraft::error::ForwardToLeader<ExampleNodeId, BasicNode>;
    pub type InitializeError = openraft::error::InitializeError<ExampleNodeId, BasicNode>;

    pub type ClientWriteResponse = openraft::raft::ClientWriteResponse<ExampleTypeConfig>;
}

pub async fn start_example_raft_node(node_id: ExampleNodeId, http_addr: String,dir: String) -> std::io::Result<()> {
    // Create a configuration for the raft instance.
    let config = Config {
        heartbeat_interval: 500,
        election_timeout_min: 1500,
        election_timeout_max: 3000,
        ..Default::default()
    };

    let config = Arc::new(config.validate().unwrap());


    let path_dir:PathBuf=dir.into();

    // Create a instance of where the Raft data will be stored.
    let store = ExampleStore::new(path_dir).await;

    // Create the network layer that will connect and communicate the raft instances and
    // will be used in conjunction with the store created above.
    let network = ExampleNetwork {};

    // Create a local raft instance.
    let raft = Raft::new(node_id, config.clone(), network, store.clone()).await.unwrap();

    // Create an application that will store all the instances created above, this will
    // be later used on the actix-web services.
    let app = Arc::new(ExampleApp {
        id: node_id,
        addr: http_addr.clone(),
        raft,
        store,
        config,
    });

    // start a tcp server
    let listener = TcpListener::bind(&http_addr).await?;
    println!("tcp Listening on: {:?}", &http_addr);

    loop {
        let app = app.clone();

        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream,app).await {
                eprintln!("failed to process client: {}", e);
            }
            //cache
        });

    }

}




async fn handle_client(mut stream: TcpStream, app :Arc<ExampleApp>) -> std::io::Result<()> {
    let mut buf = vec![0u8; 4096];
    loop {
        match stream.read(&mut buf).await {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    // Connection was closed by the client
                    break;
                }
                //println!("Received message:{:?}", &buf[..bytes_read]);
                let cmd: Command = serde_json::from_slice(&buf[..bytes_read]).unwrap();
                println!("accepted command :{:?}", &cmd);
                match cmd {
                    Command::Init => {
                        let mut nodes = BTreeMap::new();
                        nodes.insert(app.id, BasicNode::new(app.addr.clone()));
                        let res = app.raft.initialize(nodes).await;
                        match res {
                            Ok(_) => {
                                let res =serde_json::to_string(&Response::Ok).unwrap();
                                println!("init success: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                            Err(e) => {
                                let res=serde_json::to_string(&e).unwrap();
                                println!("init failed: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }

                        //res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
                    }
                    Command::Metrics => {
                        let metrics = app.raft.metrics().borrow().clone();
                        let res = serde_json::to_string(&metrics).unwrap();
                        println!("metrics: {:?}",res);
                        stream.write_all(&res.into_bytes()).await?;

                    }
                    Command::AddLeader { node_id, addr } => {
                        let node = BasicNode::new(addr.clone());
                        let res = app.raft.add_learner(node_id, node, true).await;

                        match res {
                            Ok(cwr) => {
                               let res =serde_json::to_string(&cwr).unwrap();
                                println!("add success: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                            Err(e) => {
                                //let res = bincode::serialize(&Response::Err(e.to_string())).unwrap();
                                let res=serde_json::to_string(&e).unwrap();
                                println!("add fail: {:?}",&res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }
                        //res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
                    }
                    Command::ChangeMembership { members } => {
                        let mut nodes = BTreeSet::new();
                        for node_id in members {
                            nodes.insert(node_id);
                        }
                        let res = app.raft.change_membership(nodes, false).await;
                        match res {
                            Ok(cwr) => {
                                let res =serde_json::to_string(&cwr).unwrap();
                                println!("change member success: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                            Err(e) => {
                                //let res = bincode::serialize(&Response::Err(e.to_string())).unwrap();
                                let res=serde_json::to_string(&e).unwrap();
                                println!("change member fail: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }
                        // res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
                    }
                    Command::Write { key, value } => {
                        let req = ExampleRequest::Set { key: key, value: value };
                        let res = app.raft.client_write(req).await;
                        //res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
                        match res {
                            Ok(cwr) => {
                                let res =serde_json::to_string(&cwr).unwrap();
                                println!("write success: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                            Err(e) => {
                                //let res = bincode::serialize(&Response::Err(e.to_string())).unwrap();
                                let res=serde_json::to_string(&e).unwrap();
                                println!("write fail: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }
                    }
                    Command::Read { key } => {
                        let state_machine = app.store.state_machine.read().await;
                        let res = state_machine.data.get(key);
                        match res {
                            Ok(value) => {
                                println!("read success: {:?}",&value);
                                stream.write_all(&value.into_bytes()).await?;
                            }
                            Err(e) => {
                                let res = serde_json::to_string(&Response::NotFound).unwrap();
                                println!(" read fail: {:?}",res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }

                       // res.map(|value| Response::Value(value)).unwrap_or_else(|| Response::NotFound);
                    }
                    Command::ConsistentRead { key } => {
                        let ret = app.raft.is_leader().await;
                        match ret {
                            Ok(_) => {
                                let state_machine = app.store.state_machine.read().await;
                                let res = state_machine.data.get(key);
                                match res {
                                    Ok(value) => {
                                        println!("read success: {:?}",&value);
                                        stream.write_all(&value.into_bytes()).await?;
                                    }
                                   Err(e) => {
                                        let res = serde_json::to_string(&Response::NotFound).unwrap();
                                        println!("read fail: {:?}",&res);
                                        stream.write_all(&res.into_bytes()).await?;
                                    }
                                }

                                //res.map(|value| Response::Value(value)).unwrap_or_else(|| Response::NotFound);
                            }
                            Err(e) => {
                                let res = serde_json::to_string(&e).unwrap();
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }
                    }
                    Command::Vote(votereq) => {

                        let res = app.raft.vote(votereq).await;
                        match res {
                            Ok(cwr) => {
                                let res=serde_json::to_string(&cwr).unwrap();
                                println!("vote success {:?}", &res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                            Err(e) => {
                                let res = serde_json::to_string(&e).unwrap();
                                println!("vote  fail {:?}", &e);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }
                    }
                    Command::AppendEntries(append_entries) => {
                        //let req = bincode::deserialize(&append_entries).unwrap();
                        let res = app.raft.append_entries(append_entries).await;
                        match res{
                            Ok(cwr) => {
                                let res=serde_json::to_string(&cwr).unwrap();
                                println!("append entries success  {:?}", &res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                            Err(e) => {
                                let res = serde_json::to_string(&e).unwrap();
                                println!("append entries fail {:?}", &res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }

                    }
                    Command::InstallSnapshot(install_snapshot) => {
                        //let req = bincode::deserialize(&install_snapshot).unwrap();
                        let res = app.raft.install_snapshot(install_snapshot).await;

                        match res {
                            Ok(cwr) => {
                                let res=serde_json::to_string(&cwr).unwrap();
                                println!("install snapshot success result {:?}", &res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                            Err(e) => {
                                let res = serde_json::to_string(&e).unwrap();
                                println!("install snapshot success fail {:?}", &res);
                                stream.write_all(&res.into_bytes()).await?;
                            }
                        }

                    }
                }
            }
            Err(e) => {
                println!("Error reading from socket: {}", e);
                break;
            }
        }
    }

    Ok(())
}

pub async fn runTcpServer(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    // loop {
    //     let cache = cache.clone();
    //
    //     let (stream, _) = listener.accept().await?;
    //     tokio::spawn(async move {
    //         if let Err(e) = handle_client(stream,cache).await {
    //             eprintln!("failed to process client: {}", e);
    //         }
    //         //cache
    //     });
    //
    // }
    todo!()
}
//
// async fn handle_command(cmd: Command, app :&Arc<ExampleApp>)->Response {
//     match cmd {
//         Command::Init => {
//             let mut nodes=BTreeMap::new();
//             nodes.insert(app.id, BasicNode::new(app.addr.clone()));
//             let res =app.raft.initialize(nodes).await;
//
//             res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
//         }
//         Command::Metrics => {
//             let metrics = app.raft.metrics().borrow().clone();
//             println!("{:?}--{}", metrics.id,metrics.current_term);
//             let res = bincode::serialize(&metrics).unwrap();
//
//             return Response::Metrics(res)
//
//         }
//         Command::AddLeader { node_id,addr} => {
//             let node=BasicNode::new(addr.clone());
//             let res = app.raft.add_learner(node_id,node,true).await;
//             res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
//         }
//         Command::ChangeMembership {members  } => {
//             let mut nodes=BTreeSet::new();
//             for node_id in members {
//                 nodes.insert(node_id);
//             }
//             let res = app.raft.change_membership(nodes, false).await;
//             res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
//         }
//         Command::Write { key,value} => {
//
//             let req = ExampleRequest::Set {key: key, value:value};
//             let res = app.raft.client_write(req).await;
//             res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
//         }
//         Command::Read { key} => {
//             let state_machine = app.store.state_machine.read().await;
//             let res = state_machine.data.get(&key).cloned();
//             res.map(|value| Response::Value(value)).unwrap_or_else(|| Response::NotFound);
//
//         }
//         Command::ConsistentRead { key } => {
//             let ret=app.raft.is_leader().await;
//             match ret {
//                 Ok(_) => {
//                     let state_machine = app.store.state_machine.read().await;
//                     let res = state_machine.data.get(&key).cloned();
//                     res.map(|value| Response::Value(value)).unwrap_or_else(|| Response::NotFound);
//                 }
//                 Err(e) => {
//                     return Response::Err(e.to_string())
//                 }
//             }
//         }
//         Command::Vote(vote) => {
//             let req =bincode::deserialize(&vote).unwrap();
//             let res = app.raft.vote(req).await;
//             res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
//
//         }
//         Command::AppendEntries(append_entries) => {
//             let req =bincode::deserialize(&append_entries).unwrap();
//             let res = app.raft.append_entries(req).await;
//             res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
//         }
//         Command::InstallSnapshot(install_snapshot) => {
//             let req =bincode::deserialize(&install_snapshot).unwrap();
//             let res = app.raft.install_snapshot(req).await;
//             res.map(|_| Response::Ok).unwrap_or_else(|e| Response::Err(e.to_string()));
//         }
//     }
//     Response::Ok
// }