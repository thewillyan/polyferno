use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use iceoryx2::{node, service::ipc};

use super::data::{BroadcastClient, BroadcastServer, ModelBytes, RoundId, mem::MB};
use crate::network::data::BroadcastReq;

// re-exports...
pub use super::data::NodeId;

/// An polyferno node. Performs DFL algorithms.
/// By default allocates 10MB for the model, change if necessary.
pub struct Node<const NUM_NODES: usize, const MODEL_SIZE: usize = { 10 * MB }> {
    id: NodeId,
    state: NodeState<MODEL_SIZE>,
    ice_node: node::Node<ipc::Service>,
    server: BroadcastServer<NUM_NODES, MODEL_SIZE>,
    neighbors: Vec<Neighboor<NUM_NODES, MODEL_SIZE>>,
}

impl<const NUM_NODES: usize, const MODEL_SIZE: usize> Node<NUM_NODES, MODEL_SIZE> {
    /// Run the node.
    ///
    /// (future): The node must have 2 threads (in a thread pool kind-of), one for server,
    /// to answerd requests and one for training and sending its own broadcasts.
    pub fn run(&mut self) {
        while self.ice_node.wait(Duration::from_millis(10)).is_ok() {
            while let Some(sample) = self.server.receive().unwrap() {
                let origin = sample.origin;
                let src = sample.user_header().src;
                let model_bytes = sample.model_bytes.clone();

                // Add broadcasted bytes to node's own knowledge.
                self.state.inbox.insert(origin, model_bytes);

                // Add the origin node to the source of the broadcast.
                if src != origin {
                    let neigh = self.neighbors.iter_mut().find(|n| n.id == src).unwrap();
                    neigh.knowledge.insert(origin);
                }

                // Broadcast bytes
                self.broadcast(sample.origin);
            }
        }
        unimplemented!()
    }

    /// Propagates the broadcast to the neighbors.
    fn broadcast(&mut self, origin: NodeId) {
        let model_bytes = if origin == self.id {
            self.model_bytes()
        } else {
            let model_bytes: &ModelBytes<MODEL_SIZE> = self
                .state
                .inbox
                .get(&origin)
                .expect("Should first receive the broadcast before fowarding it");
            model_bytes.clone()
        };

        let bcast_req = BroadcastReq {
            origin,
            round: self.state.round,
            model_bytes,
        };
        for n in &self.neighbors {
            // skip broadcast if the node already knows origin data
            let already_knows = n.id == origin || n.knowledge.iter().any(|&id| id == origin);
            if already_knows {
                continue;
            }

            // broadcast data to neighboor
            let mut sample = n.client.loan_uninit().unwrap();
            let header = sample.user_header_mut();
            header.src = self.id;
            header.dst = n.id;

            let sample = sample.write_payload(bcast_req.clone());
            sample.send().unwrap();
        }
    }

    /// Retrieve the model bytes.
    fn model_bytes(&mut self) -> ModelBytes<MODEL_SIZE> {
        let mut bytes = ModelBytes::new();
        bytes.extend_from_slice(&self.state.model_bytes);
        bytes
    }
}

/// Stores the state of an node in a given communication round.
struct NodeState<const MODEL_SIZE: usize> {
    round: RoundId,
    inbox: HashMap<NodeId, ModelBytes<MODEL_SIZE>>,
    model_bytes: Vec<u8>,
}

/// A neighboor of an DFL node.
struct Neighboor<const NUM_NODES: usize, const MODEL_SIZE: usize> {
    id: NodeId,
    client: BroadcastClient<NUM_NODES, MODEL_SIZE>,
    knowledge: HashSet<NodeId>,
}
