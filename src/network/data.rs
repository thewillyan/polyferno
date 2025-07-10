use iceoryx2::{
    port::{client::Client, server::Server},
    prelude::ZeroCopySend,
    service::ipc,
};
use iceoryx2_bb_container::vec::FixedSizeVec;

/// Type of a communication round identifier.
pub type RoundId = u64;
/// Type of a node identifier.
pub type NodeId = u64;
/// A shared-memory compatible version of the `bincode` byte representation of a `burn` model.
pub type ModelBytes<const MODEL_SIZE: usize> = FixedSizeVec<u8, MODEL_SIZE>;

/// A iceoryx2 client who sends `BroadcastReq` requests and receives `BroadcastResp`.
pub type BroadcastClient<const NUM_NODES: usize, const MODEL_SIZE: usize> = Client<
    ipc::Service,
    BroadcastReq<MODEL_SIZE>,
    SimpleHeader,
    BroadcastResp<NUM_NODES>,
    SimpleHeader,
>;

/// A iceoryx2 server who responds to `BroadcastReq` requests with `BroadcastResp`.
pub type BroadcastServer<const NUM_NODES: usize, const MODEL_SIZE: usize> = Server<
    ipc::Service,
    BroadcastReq<MODEL_SIZE>,
    SimpleHeader,
    BroadcastResp<NUM_NODES>,
    SimpleHeader,
>;

pub mod mem {
    /// A kilobyte in bytes.
    pub const KB: usize = 1024;
    /// A megabyte in bytes.
    pub const MB: usize = 1024 * KB;
    /// A gigabyte in bytes.
    pub const GB: usize = 1024 * MB;
}

/// A common header used by eather requests or responses.
///
/// Fields:
/// - `src`: the source of the message
/// - `dst`: the destination of the message
#[derive(ZeroCopySend, Debug, Clone)]
#[repr(C)]
pub struct SimpleHeader {
    pub src: NodeId,
    pub dst: NodeId,
}

/// Requests the `model_bytes` of the `round` should be propagated to ever other `Node`.
///
/// Fields:
/// - `model_bytes`: the `bincode` formated bytes of the model.
/// - `origin`: the id of the node where the broadcast originally come from.
/// - `round`: the last round that the model was trained.
#[derive(ZeroCopySend, Debug, Clone)]
#[repr(C)]
pub struct BroadcastReq<const MODEL_SIZE: usize> {
    pub origin: NodeId,
    pub round: RoundId,
    pub model_bytes: ModelBytes<MODEL_SIZE>,
}

/// Responds a `BroadcastReq` with the models the `Node` knows for the communication round.
///
/// Fields:
/// - `knowledge`: an list of `NodeId`'s. An id being on this list means that the source of the
/// message knows the model of the node with this id.
/// - `round`: the round which your knowledge is based on.
#[derive(ZeroCopySend, Debug, Clone)]
#[repr(C)]
pub struct BroadcastResp<const NUM_NODES: usize> {
    pub round: RoundId,
    pub knowledge: FixedSizeVec<NodeId, NUM_NODES>,
}
