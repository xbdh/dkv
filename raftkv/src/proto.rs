use serde::{Deserialize, Serialize};
use openraft::raft::VoteRequest;
use openraft::raft::AppendEntriesRequest;
use openraft::raft::InstallSnapshotRequest;
use openraft::raft::AppendEntriesResponse;
use openraft::raft::InstallSnapshotResponse;
use openraft::raft::VoteResponse;
use openraft::{LeaderId, Vote};
use crate::ExampleTypeConfig;
use crate::ExampleNodeId;

#[derive(Debug,Clone,Deserialize,Serialize)]
pub enum Command{
    Init,
    Metrics,
    AddLeader { node_id:u64,addr:String},
    ChangeMembership {members :Vec<u64>},
    Write{key:String,value:String},
    Read{key:String},
    ConsistentRead {key:String},

    Vote(VoteRequest<ExampleNodeId>),
    AppendEntries(AppendEntriesRequest<ExampleTypeConfig>),
    InstallSnapshot(InstallSnapshotRequest<ExampleTypeConfig>),
}
#[derive(Debug,Clone,Deserialize,Serialize)]
pub enum Response{
    Ok,
    Err(String),
    NotFound,
    Value(String),
    Metrics(Vec<u8>),
}