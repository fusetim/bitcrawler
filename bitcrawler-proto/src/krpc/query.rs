use std::collections::HashMap;

use crate::{
    bencode::{BencodeDict, BencodeString, BencodeValue},
    kademlia::NodeId,
};

use super::{ToArguments, TryFromArguments, TryFromArgumentsError};

/// Query type associated for the `ping` query.
pub const QUERY_TYPE_PING: &[u8] = b"ping";
/// Query type associated for the `find_node` query.
pub const QUERY_TYPE_FIND_NODE: &[u8] = b"find_node";
/// Query type associated for the `get_peers` query.
pub const QUERY_TYPE_GET_PEERS: &[u8] = b"get_peers";
/// Query type associated for the `announce_peer` query.
pub const QUERY_TYPE_ANNOUNCE_PEER: &[u8] = b"announce_peer";

/// Represents a query message in the KRPC protocol.
///
/// More information about the KRPC protocol can be found in the [specification](https://www.bittorrent.org/beps/bep_0005.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Query<N: NodeId> {
    transaction_id: BencodeString,
    query: QueryType<N>,
}

/// Represents a query type in the KRPC protocol.
///
/// Only 4 query types are supported: `ping`, `find_node`, `get_peers`, and `announce_peer`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum QueryType<N: NodeId> {
    /// Represents a `ping` query.
    Ping(Ping<N>),
    /// Represents a `find_node` query.
    FindNode(FindNode<N>),
    /// Represents a `get_peers` query.
    GetPeers(GetPeers<N>),
    /// Represents an `announce_peer` query.
    AnnouncePeer(AnnouncePeer<N>),
}

/// Represents a `ping` query in the KRPC protocol.
///
/// The `ping` query is used to check if a node is still alive.
/// The only argument required for a `ping` query is the `id` of the node.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ping<N: NodeId> {
    id: N,
}

/// Represents a `find_node` query in the KRPC protocol.
///
/// The `find_node` query is used to find the `k` nodes closest to a given target node.
/// The arguments required for a `find_node` query are the `id` of the node and the `target` node.
/// The `target` node is the node whose neighbors are being searched for.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FindNode<N: NodeId> {
    id: N,
    target: N,
}

/// Represents a `get_peers` query in the KRPC protocol.
///
/// The `get_peers` query is used to find peers that are downloading a specific torrent.
/// The arguments required for a `get_peers` query are the `id` of the node and the `info_hash` of the torrent.
/// The `info_hash` is the SHA-1 hash of the metadata of the torrent.
/// The response to a `get_peers` query will contain a list of peers that are downloading the torrent.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GetPeers<N: NodeId> {
    id: N,
    info_hash: N,
}

/// Represents an `announce_peer` query in the KRPC protocol.
///
/// The `announce_peer` query is used to announce that the node is downloading a specific torrent.
/// The arguments required for an `announce_peer` query are the `id` of the node, the `info_hash` of the torrent,
/// the `port` on which the node is downloading the torrent, and a `token` received from a previous `get_peers` query.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnnouncePeer<N: NodeId> {
    id: N,
    info_hash: N,
    port: u16,
    token: BencodeString,
}

impl<N: NodeId> Query<N> {
    pub fn new(transaction_id: impl Into<BencodeString>, query: QueryType<N>) -> Self {
        Query {
            transaction_id: transaction_id.into(),
            query,
        }
    }

    pub fn to_bencoded(&self) -> BencodeValue {
        let mut dictionary = HashMap::new();
        dictionary.insert(
            "t".into(),
            BencodeValue::ByteString(self.transaction_id.clone().into()),
        );
        dictionary.insert("y".into(), BencodeValue::ByteString("q".into()));
        dictionary.insert(
            "q".into(),
            BencodeValue::ByteString(self.query.get_query_type().into()),
        );
        dictionary.insert(
            "a".into(),
            BencodeValue::Dict(self.query.to_arguments().into_iter().collect()),
        );
        BencodeValue::Dict(dictionary.into_iter().collect())
    }

    pub fn try_from_bencoded(input: &BencodeValue) -> Result<Self, TryFromArgumentsError> {
        let dict = match input {
            BencodeValue::Dict(dict) => dict,
            _ => return Err("Invalid query - not a dictionary"),
        };

        let transaction_id = match dict.iter().find(|(key, _)| key.as_ref() == b"t") {
            Some((_, BencodeValue::ByteString(transaction_id))) => transaction_id.clone(),
            _ => return Err("Missing 't' field"),
        };
        let query_type = match dict.iter().find(|(key, _)| key.as_ref() == b"q") {
            Some((_, BencodeValue::ByteString(query_type))) => query_type,
            _ => return Err("Missing 'q' field"),
        };
        let arguments = match dict.iter().find(|(key, _)| key.as_ref() == b"a") {
            Some((_, BencodeValue::Dict(arguments))) => arguments,
            _ => return Err("Missing 'a' field"),
        };

        let query = match query_type.as_ref() {
            QUERY_TYPE_PING => QueryType::Ping(Ping::try_from_arguments(arguments)?),
            QUERY_TYPE_FIND_NODE => QueryType::FindNode(FindNode::try_from_arguments(arguments)?),
            QUERY_TYPE_GET_PEERS => QueryType::GetPeers(GetPeers::try_from_arguments(arguments)?),
            QUERY_TYPE_ANNOUNCE_PEER => {
                QueryType::AnnouncePeer(AnnouncePeer::try_from_arguments(arguments)?)
            }
            _ => return Err("Invalid query type"),
        };

        Ok(Query::new(transaction_id, query))
    }
}

impl<N: NodeId> QueryType<N> {
    pub fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        match self {
            QueryType::Ping(ping) => ping.to_arguments(),
            QueryType::FindNode(find_node) => find_node.to_arguments(),
            QueryType::GetPeers(get_peers) => get_peers.to_arguments(),
            QueryType::AnnouncePeer(announce_peer) => announce_peer.to_arguments(),
        }
    }

    pub fn get_query_type(&self) -> &[u8] {
        match self {
            QueryType::Ping(_) => QUERY_TYPE_PING,
            QueryType::FindNode(_) => QUERY_TYPE_FIND_NODE,
            QueryType::GetPeers(_) => QUERY_TYPE_GET_PEERS,
            QueryType::AnnouncePeer(_) => QUERY_TYPE_ANNOUNCE_PEER,
        }
    }
}

impl<N: NodeId> ToArguments for Ping<N> {
    fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        let mut arguments = HashMap::new();
        let id: Vec<u8> = self.id.clone().into();
        arguments.insert("id".into(), BencodeValue::ByteString(id.into()));
        arguments
    }
}

impl<N: NodeId> ToArguments for FindNode<N> {
    fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        let mut arguments = HashMap::new();
        let id: Vec<u8> = self.id.clone().into();
        let target: Vec<u8> = self.target.clone().into();
        arguments.insert("id".into(), BencodeValue::ByteString(id.into()));
        arguments.insert("target".into(), BencodeValue::ByteString(target.into()));
        arguments
    }
}

impl<N: NodeId> ToArguments for GetPeers<N> {
    fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        let mut arguments = HashMap::new();
        let id: Vec<u8> = self.id.clone().into();
        let info_hash: Vec<u8> = self.info_hash.clone().into();
        arguments.insert("id".into(), BencodeValue::ByteString(id.into()));
        arguments.insert(
            "info_hash".into(),
            BencodeValue::ByteString(info_hash.into()),
        );
        arguments
    }
}

impl<N: NodeId> ToArguments for AnnouncePeer<N> {
    fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        let mut arguments = HashMap::new();
        let id: Vec<u8> = self.id.clone().into();
        let info_hash: Vec<u8> = self.info_hash.clone().into();
        arguments.insert("id".into(), BencodeValue::ByteString(id.into()));
        arguments.insert(
            "info_hash".into(),
            BencodeValue::ByteString(info_hash.into()),
        );
        arguments.insert("port".into(), BencodeValue::Integer(self.port as i128));
        arguments.insert("token".into(), BencodeValue::ByteString(self.token.clone()));
        arguments
    }
}

impl<N: NodeId> TryFromArguments for Ping<N> {
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key.as_ref() == b"id")
            .ok_or("Missing 'id' field")?;
        if let BencodeValue::ByteString(id) = id {
            Ok(Ping {
                id: N::try_from(id.as_ref()).or(Err("Invalid NodeId"))?,
            })
        } else {
            Err("Invalid 'id' field")
        }
    }
}

impl<N: NodeId> TryFromArguments for FindNode<N> {
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key.as_ref() == b"id")
            .ok_or("Missing 'id' field")?;
        let (_, target) = arguments
            .iter()
            .find(|(key, _)| key.as_ref() == b"target")
            .ok_or("Missing 'target' field")?;
        if let (BencodeValue::ByteString(id), BencodeValue::ByteString(target)) = (id, target) {
            Ok(FindNode {
                id: N::try_from(id.as_ref()).or(Err("Invalid NodeId"))?,
                target: N::try_from(target.as_ref()).or(Err("Invalid NodeId"))?,
            })
        } else {
            Err("Invalid 'id' or 'target' field")
        }
    }
}

impl<N: NodeId> TryFromArguments for GetPeers<N> {
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key.as_ref() == b"id")
            .ok_or("Missing 'id' field")?;
        let (_, info_hash) = arguments
            .iter()
            .find(|(key, _)| key.as_ref() == b"info_hash")
            .ok_or("Missing 'info_hash' field")?;
        if let (BencodeValue::ByteString(id), BencodeValue::ByteString(info_hash)) = (id, info_hash)
        {
            Ok(GetPeers {
                id: N::try_from(id.as_ref()).or(Err("Invalid NodeId"))?,
                info_hash: N::try_from(info_hash.as_ref()).or(Err("Invalid NodeId/InfoHash"))?,
            })
        } else {
            Err("Invalid 'id' or 'info_hash' field")
        }
    }
}

impl<N: NodeId> TryFromArguments for AnnouncePeer<N> {
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError> {
        let (mut id, mut info_hash, mut port, mut token) = (None, None, None, None);
        for (key, value) in arguments {
            match key.as_ref() {
                b"id" => {
                    if let BencodeValue::ByteString(id_) = value {
                        id = Some(N::try_from(id_.as_ref()).or(Err("Invalid NodeId"))?);
                    } else {
                        return Err("Invalid 'id' field");
                    }
                }
                b"info_hash" => {
                    if let BencodeValue::ByteString(info_hash_) = value {
                        info_hash =
                            Some(N::try_from(info_hash_.as_ref()).or(Err("Invalid InfoHash"))?);
                    } else {
                        return Err("Invalid 'info_hash' field");
                    }
                }
                b"port" => {
                    if let BencodeValue::Integer(port_) = value {
                        if *port_ < 0 || *port_ > u16::MAX as i128 {
                            return Err("Invalid 'port' field");
                        }
                        port = Some(*port_ as u16);
                    } else {
                        return Err("Invalid 'port' field");
                    }
                }
                b"token" => {
                    if let BencodeValue::ByteString(token_) = value {
                        token = Some(token_.clone());
                    } else {
                        return Err("Invalid 'token' field");
                    }
                }
                _ => { /* Ignore */ }
            }
        }
        match (id, info_hash, port, token) {
            (Some(id), Some(info_hash), Some(port), Some(token)) => Ok(AnnouncePeer {
                id,
                info_hash,
                port,
                token,
            }),
            _ => Err("Missing required field(s)"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockNodeId;
    use super::*;

    #[test]
    fn test_ping_query_to_bencoded() {
        let node_id = MockNodeId::try_from(&b"25000000"[..]).unwrap();
        let node_id_: Vec<u8> = node_id.clone().into();

        let query = Query::new(
            "transaction_id",
            QueryType::Ping(Ping {
                id: node_id.clone(),
            }),
        );
        let mut bencoded = query.to_bencoded();
        let mut expected = BencodeValue::Dict(
            vec![
                (
                    "t".into(),
                    BencodeValue::ByteString("transaction_id".into()),
                ),
                ("y".into(), BencodeValue::ByteString("q".into())),
                ("q".into(), BencodeValue::ByteString("ping".into())),
                (
                    "a".into(),
                    BencodeValue::Dict(
                        vec![("id".into(), BencodeValue::ByteString(node_id_.into()))]
                            .into_iter()
                            .collect(),
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        );
        bencoded.sort_keys();
        expected.sort_keys();
        assert_eq!(bencoded, expected);
    }
}
