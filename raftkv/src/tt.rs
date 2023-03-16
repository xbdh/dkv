use serde::{Deserialize, Serialize};
use std::io::{Result, Write};

use openraft::raft::VoteRequest;
use openraft::raft::AppendEntriesRequest;
use openraft::raft::InstallSnapshotRequest;
use openraft::raft::AppendEntriesResponse;
use openraft::raft::InstallSnapshotResponse;
use openraft::raft::VoteResponse;
use openraft::{LeaderId, Vote};
use raftkv::ExampleNodeId;
use raftkv::ExampleTypeConfig;
//use raft_kv_memstore::proto::{Command, Response};

#[derive(Debug,Clone,Deserialize,Serialize)]
enum Command{
    Init,
    Metrics,
    AddLeader { node_id:u64,addr:String},
    ChangeMembership {node_id:Vec<u64>},

    Write{key:Vec<u8>,value:Vec<u8>},
    Read{key:Vec<u8>},
    ConsistentRead {key:Vec<u8>},
    vote(VoteRequest<ExampleNodeId>),
}




fn main() -> Result<()> {

    // //let node_id:ExampleNodeId=1;
    //
    // let cmd=Command::vote(VoteRequest{
    //     vote: Vote {
    //         leader_id: LeaderId {
    //             term: 0,
    //             node_id: 1,
    //         },
    //
    //         committed: false,
    //     },
    //
    //     last_log_id: None,
    // });
    // let ss:Result<u64>=Ok(1);
    //
    //
    //
    // println!("JSON length: {} -{}", json_len,&json);


    Ok(())
}
