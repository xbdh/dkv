use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::RangeBounds;
use std::sync::Arc;
use std::sync::Mutex;
use std::path::{Path, PathBuf};

use openraft::async_trait::async_trait;
use openraft::storage::LogState;
use openraft::storage::Snapshot;
use openraft::AnyError;
use openraft::BasicNode;
use openraft::Entry;
use openraft::EntryPayload;
use openraft::ErrorSubject;
use openraft::ErrorVerb;
use openraft::LogId;
use openraft::RaftLogReader;
use openraft::RaftSnapshotBuilder;
use openraft::RaftStorage;
use openraft::SnapshotMeta;
use openraft::StorageError;
use openraft::StorageIOError;
use openraft::StoredMembership;
use openraft::Vote;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::ExampleNodeId;
use crate::ExampleTypeConfig;

use bitcask::db::{Engine};
use bitcask::options;
use bitcask::options::Options;

/**
 * Here you will set the types of request that will interact with the raft nodes.
 * For example the `Set` will be used to write data (key and value) to the raft database.
 * The `AddNode` will append a new node to the current existing shared list of nodes.
 * You will want to add any request that can write data in all nodes here.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExampleRequest {
    Set { key: String, value: String },
}

/**
 * Here you will defined what type of answer you expect from reading the data of a node.
 * In this example it will return a optional value from a given key in
 * the `ExampleRequest.Set`.
 *
 * TODO: SHould we explain how to create multiple `AppDataResponse`?
 *
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExampleResponse {
    pub value: Option<String>,
}

#[derive(Debug)]
pub struct ExampleSnapshot {
    pub meta: SnapshotMeta<ExampleNodeId, BasicNode>,

    /// The data of the state machine at the time of this snapshot.
    pub data: Vec<u8>,
}

/**
 * Here defines a state machine of the raft, this state represents a copy of the data
 * between each node. Note that we are using `serde` to serialize the `data`, which has
 * a implementation to be serialized. Note that for this test we set both the key and
 * value as String, but you could set any type of value that has the serialization impl.
 */
#[derive(Debug, Clone,Default)]
pub struct ExampleStateMachine {
    pub last_applied_log: Option<LogId<ExampleNodeId>>,

    pub last_membership: StoredMembership<ExampleNodeId, BasicNode>,

    /// Application data.
    //pub data: BTreeMap<String, String>,
     pub data :Engine,// 自定义的bitcask引擎
}

impl ExampleStateMachine {
    pub fn new(data :Engine) -> Self {
        ExampleStateMachine {
            //data: BTreeMap::new(),
            data:data,
            ..Default::default()
        }
    }
}

pub struct MYexampleStateMachine {
    /// Application data.
    //pub data: BTreeMap<String, String>,
   pub data :Engine,// 自定义的bitcask引擎
}

// impl From<&ExampleStateMachine> for MYexampleStateMachine{
//     fn from(state_machine: &ExampleStateMachine) -> Self {
//         let ops=options::Options {
//             dir_path: "./store".to_string().parse().unwrap(),
//             max_segment_size: 100,
//             sync_write: true,
//         };
//         let mut log= bitcask::db::Engine::new(ops).unwrap();
//         for (key,value) in state_machine.data.iter(){
//             log.append(Vec::from(key.as_bytes()), Vec::from(value.as_bytes())).unwrap();
//         }
//         MYexampleStateMachine{
//             data:log,
//         }
//     }
// }

#[derive(Debug, Default)]
pub struct ExampleStore {
    last_purged_log_id: RwLock<Option<LogId<ExampleNodeId>>>,

    /// The Raft log.
    log: RwLock<BTreeMap<u64, Entry<ExampleTypeConfig>>>,

    /// The Raft state machine.
    pub state_machine: RwLock<ExampleStateMachine>,

    //The current granted vote.
    vote: RwLock<Option<Vote<ExampleNodeId>>>,
//
    snapshot_idx: Arc<Mutex<u64>>,

    current_snapshot: RwLock<Option<ExampleSnapshot>>,

}



#[async_trait]
impl RaftLogReader<ExampleTypeConfig> for Arc<ExampleStore> {
    async fn get_log_state(&mut self) -> Result<LogState<ExampleTypeConfig>, StorageError<ExampleNodeId>> {
        let log = self.log.read().await;
        let last = log.iter().rev().next().map(|(_, ent)| ent.log_id);

        let last_purged = *self.last_purged_log_id.read().await;

        let last = match last {
            None => last_purged,
            Some(x) => Some(x),
        };

        Ok(LogState {
            last_purged_log_id: last_purged,
            last_log_id: last,
        })

        //my change
        //todo!()
    }

    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + Send + Sync>(
        &mut self,
        range: RB,
    ) -> Result<Vec<Entry<ExampleTypeConfig>>, StorageError<ExampleNodeId>> {
        let log = self.log.read().await;
        let response = log.range(range.clone()).map(|(_, val)| val.clone()).collect::<Vec<_>>();
        Ok(response)
    }
}

#[async_trait]
impl RaftSnapshotBuilder<ExampleTypeConfig, Cursor<Vec<u8>>> for Arc<ExampleStore> {
    #[tracing::instrument(level = "trace", skip(self))]
    async fn build_snapshot(
        &mut self,
    ) -> Result<Snapshot<ExampleNodeId, BasicNode, Cursor<Vec<u8>>>, StorageError<ExampleNodeId>> {
        let data;
        let last_applied_log;
        let last_membership;

        {
            //Serialize the data of the state machine.
            let state_machine = self.state_machine.read().await;
            // data = serde_json::to_vec(&*state_machine)
            //     .map_err(|e| StorageIOError::new(ErrorSubject::StateMachine, ErrorVerb::Read, AnyError::new(&e)))?;

            //my change
            data=vec![1,2,3,4,5,6,7,8,9,10];
            //my change


            last_applied_log = state_machine.last_applied_log;
            last_membership = state_machine.last_membership.clone();
        }

        let snapshot_idx = {
            let mut l = self.snapshot_idx.lock().unwrap();
            *l += 1;
            *l
        };

        let snapshot_id = if let Some(last) = last_applied_log {
            format!("{}-{}-{}", last.leader_id, last.index, snapshot_idx)
        } else {
            format!("--{}", snapshot_idx)
        };

        let meta = SnapshotMeta {
            last_log_id: last_applied_log,
            last_membership,
            snapshot_id,
        };

        let snapshot = ExampleSnapshot {
            meta: meta.clone(),
            data: data.clone(),
        };

        {
            let mut current_snapshot = self.current_snapshot.write().await;
            *current_snapshot = Some(snapshot);
        }

        Ok(Snapshot {
            meta,
            snapshot: Box::new(Cursor::new(data)),
        })
        // OK(Snapshot::Default)
        // //todo!()
    }
}

#[async_trait]
impl RaftStorage<ExampleTypeConfig> for Arc<ExampleStore> {
    type SnapshotData = Cursor<Vec<u8>>; //can Chage it  to () ,my change
    type LogReader = Self;
    type SnapshotBuilder = Self;

    #[tracing::instrument(level = "trace", skip(self))]
    async fn save_vote(&mut self, vote: &Vote<ExampleNodeId>) -> Result<(), StorageError<ExampleNodeId>> {
        let mut v = self.vote.write().await;
        *v = Some(*vote);
        Ok(())
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<ExampleNodeId>>, StorageError<ExampleNodeId>> {
        Ok(*self.vote.read().await)
    }

    #[tracing::instrument(level = "trace", skip(self, entries))]
    async fn append_to_log(
        &mut self,
        entries: &[&Entry<ExampleTypeConfig>],
    ) -> Result<(), StorageError<ExampleNodeId>> {
        let mut log = self.log.write().await;
        for entry in entries {
            log.insert(entry.log_id.index, (*entry).clone());
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_conflict_logs_since(
        &mut self,
        log_id: LogId<ExampleNodeId>,
    ) -> Result<(), StorageError<ExampleNodeId>> {
        tracing::debug!("delete_log: [{:?}, +oo)", log_id);

        let mut log = self.log.write().await;
        let keys = log.range(log_id.index..).map(|(k, _v)| *k).collect::<Vec<_>>();
        for key in keys {
            log.remove(&key);
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn purge_logs_upto(&mut self, log_id: LogId<ExampleNodeId>) -> Result<(), StorageError<ExampleNodeId>> {
        tracing::debug!("delete_log: [{:?}, +oo)", log_id);

        {
            let mut ld = self.last_purged_log_id.write().await;
            assert!(*ld <= Some(log_id));
            *ld = Some(log_id);
        }

        {
            let mut log = self.log.write().await;

            let keys = log.range(..=log_id.index).map(|(k, _v)| *k).collect::<Vec<_>>();
            for key in keys {
                log.remove(&key);
            }
        }

        Ok(())
    }

    async fn last_applied_state(
        &mut self,
    ) -> Result<(Option<LogId<ExampleNodeId>>, StoredMembership<ExampleNodeId, BasicNode>), StorageError<ExampleNodeId>>
    {
        let state_machine = self.state_machine.read().await;
        Ok((state_machine.last_applied_log, state_machine.last_membership.clone()))
    }

    #[tracing::instrument(level = "trace", skip(self, entries))]
    async fn apply_to_state_machine(
        &mut self,
        entries: &[&Entry<ExampleTypeConfig>],
    ) -> Result<Vec<ExampleResponse>, StorageError<ExampleNodeId>> {
        let mut res = Vec::with_capacity(entries.len());

        let mut sm = self.state_machine.write().await;

        for entry in entries {
            tracing::debug!(%entry.log_id, "replicate to sm");

            sm.last_applied_log = Some(entry.log_id);

            match entry.payload {
                EntryPayload::Blank => res.push(ExampleResponse { value: None }),
                EntryPayload::Normal(ref req) => match req {
                    ExampleRequest::Set { key, value } => {
                        //sm.data.
                        sm.data.append(key.clone(), value.clone());
                        res.push(ExampleResponse {
                            value: Some(value.clone()),
                        })
                    }
                },
                EntryPayload::Membership(ref mem) => {
                    sm.last_membership = StoredMembership::new(Some(entry.log_id), mem.clone());
                    res.push(ExampleResponse { value: None })
                }
            };
        }
        Ok(res)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn begin_receiving_snapshot(&mut self) -> Result<Box<Self::SnapshotData>, StorageError<ExampleNodeId>> {
        //Ok(Box::new(Cursor::new(Vec::new())))
        //todo!()
        Ok(
            Box::new(Cursor::new(Vec::new()))
        )
    }

    #[tracing::instrument(level = "trace", skip(self, snapshot))]
    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<ExampleNodeId, BasicNode>,
        snapshot: Box<Self::SnapshotData>,
    ) -> Result<(), StorageError<ExampleNodeId>> {
        tracing::info!(
            { snapshot_size = snapshot.get_ref().len() },
            "decoding snapshot for installation"
        );

        let new_snapshot = ExampleSnapshot {
            meta: meta.clone(),
            data: snapshot.into_inner(),
        };

        // Update the state machine.
        {
            // let updated_state_machine: ExampleStateMachine =
            //     serde_json::from_slice(&new_snapshot.data).map_err(|e| {
            //         StorageIOError::new(
            //             ErrorSubject::Snapshot(new_snapshot.meta.signature()),
            //             ErrorVerb::Read,
            //             AnyError::new(&e),
            //         )
            //     })?;

            //my change
            let updated_state_machine: ExampleStateMachine =ExampleStateMachine::default();
            //my change

            let mut state_machine = self.state_machine.write().await;
            *state_machine = updated_state_machine;
        }

        // Update current snapshot.
        let mut current_snapshot = self.current_snapshot.write().await;
        *current_snapshot = Some(new_snapshot);

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<ExampleNodeId, BasicNode, Self::SnapshotData>>, StorageError<ExampleNodeId>> {
        match &*self.current_snapshot.read().await {
            Some(snapshot) => {
                let data = snapshot.data.clone();
                Ok(Some(Snapshot {
                    meta: snapshot.meta.clone(),
                    snapshot: Box::new(Cursor::new(data)),
                }))
            }
            None => Ok(None),
        }
        //Ok(None)
        //todo!()
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
        //todo!()
    }
}
impl ExampleStore {
    pub(crate) async fn new(db_path: PathBuf) -> Arc<ExampleStore> {
        let eg=bitcask::db::Engine::new(bitcask::options::Options {
            // dir_path: "./store".to_string().parse().unwrap(),
            dir_path: db_path,
            max_segment_size: 1024*1024*1024,
            sync_write: true,
        }).unwrap();
        
        let state_machine = RwLock::new(ExampleStateMachine::new(eg));
        Arc::new(ExampleStore {
            last_purged_log_id: Default::default(),
            log: Default::default(),
            state_machine,
            vote: Default::default(),
            snapshot_idx: Arc::new(Mutex::new(0)),
            current_snapshot: Default::default(),
        })
    }
}