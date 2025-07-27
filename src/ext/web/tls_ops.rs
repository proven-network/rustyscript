use deno_core::{op2, serde_json};

/// Stub implementation of op_tls_peer_certificate
/// This is needed because deno_net expects this op from deno_node
/// Returns None to indicate no peer certificate is available
#[op2]
#[serde]
pub fn op_tls_peer_certificate(#[smi] _rid: u32, _detailed: bool) -> Option<serde_json::Value> {
    None
}
