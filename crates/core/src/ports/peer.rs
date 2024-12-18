use async_trait::async_trait;

use crate::errors::CoreError;

/// Defines an asynchronous interface for interacting with peers.
///
/// The `PeerPort` trait allows for requesting decryption shares from peer nodes.
#[async_trait]
pub trait PeerPort {
    /// Requests a decryption share from a specified peer.
    ///
    /// This method communicates with a peer node to retrieve its contribution (decryption share),
    /// which can be used in the decryption process during multi-party computations.
    ///
    /// # Parameters
    /// - `peer_endpoint`: The network address or endpoint of the peer node.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)`: If the decryption share is successfully retrieved as a byte vector.
    /// - `Err(CoreError)`: If the request fails due to network errors, invalid responses, or other issues.
    async fn get_dec_share(&self, peer_endpoint: &str) -> Result<Vec<u8>, CoreError>;
}
