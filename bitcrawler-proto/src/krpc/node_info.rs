use crate::kademlia::NodeId;

/// Node Info represents a discovered node (id, address, port) in the network.
pub trait NodeInfo: PartialEq + Eq + Clone {
    /// The type of the node id.
    type NodeId: NodeId;
    /// The type of the address.
    type Address;

    /// Returns the node id.
    fn get_node_id(&self) -> &Self::NodeId;
    /// Returns the address of the node.
    ///
    /// The address must be sufficient to establish a connection with the node.
    /// Therefore for a TCP connection, the address is the IP address of the node and the port number.
    /// For other types of connections, the address may include additional information (WebTransport might need an additional certificate hash).
    ///
    fn to_address(&self) -> Self::Address;
    /// Creates a new instance of `NodeInfo` with the given node id and address.
    fn new_with_address(node_id: Self::NodeId, address: Self::Address) -> Self;
}

/// A trait for compact node info (must implement a way to encode/decode it)
///
/// The compact node info format is a string representation of a node info that is used in the KRPC protocol.
///
/// For BitTorrent, the compact node info format is a string of the form `<node_id><ip><port>`.
/// - `node_id` is the node id of the node (20 bytes).
/// - `ip` is the IP address of the node (4 bytes for IPv4, 16 bytes for IPv6).
/// - `port` is the port number of the node (2 bytes).
pub trait CompactNodeInfo: NodeInfo {
    type Error;
    /// Reads a compact node info from a string.
    ///
    /// # Parameters
    ///
    /// - `data`: The string containing the compact node info.
    ///
    /// # Returns
    ///
    /// A tuple containing the number of bytes read and the compact node info if successful.
    /// Otherwise, an error is returned.
    ///
    /// # Errors
    ///
    /// An error is returned if the string does not contain a valid compact node info.
    fn try_read_compact_node_info(data: &[u8]) -> Result<(usize, Self), Self::Error>;

    /// Produces a compact node info from the given node info.
    ///
    /// # Returns
    ///
    /// A string (CoW) containing the compact node info.
    fn write_compact_node_info(&self) -> Vec<u8>;
}

/// A typical IPv4 implementation of `NodeInfo` for a node in the KRPC protocol.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BittorrentNodeInfoV4<N: NodeId> {
    pub node_id: N,
    pub ip: [u8; 4],
    pub port: u16,
}

/// A typical IPv6 implementation of `NodeInfo` for a node in the KRPC protocol.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BittorrentNodeInfoV6<N: NodeId> {
    pub node_id: N,
    pub ip: [u8; 16],
    pub port: u16,
}
