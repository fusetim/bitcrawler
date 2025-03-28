use std::collections::HashMap;

use crate::{
    bencode::{BencodeDict, BencodeValue},
    kademlia::NodeId,
};

use super::{ToArguments, TryFromArguments, TryFromArgumentsError};

/// Query type associated for the `ping` query.
pub const QUERY_TYPE_PING: &str = "ping";
/// Query type associated for the `find_node` query.
pub const QUERY_TYPE_FIND_NODE: &str = "find_node";
/// Query type associated for the `get_peers` query.
pub const QUERY_TYPE_GET_PEERS: &str = "get_peers";
/// Query type associated for the `announce_peer` query.
pub const QUERY_TYPE_ANNOUNCE_PEER: &str = "announce_peer";

/// Represents a query message in the KRPC protocol.
///
/// More information about the KRPC protocol can be found in the [specification](https://www.bittorrent.org/beps/bep_0005.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Query<N: NodeId> {
    transaction_id: String,
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
    token: String,
}

impl<N: NodeId> Query<N> {
    pub fn new(transaction_id: String, query: QueryType<N>) -> Self {
        Query {
            transaction_id,
            query,
        }
    }

    pub fn to_bencoded(&self) -> BencodeValue {
        let mut dictionary = HashMap::new();
        dictionary.insert(
            "t".to_string(),
            BencodeValue::String(self.transaction_id.clone()),
        );
        dictionary.insert("y".to_string(), BencodeValue::String("q".to_string()));
        dictionary.insert(
            "q".to_string(),
            BencodeValue::String(self.query.get_query_type().to_string()),
        );
        dictionary.insert(
            "a".to_string(),
            BencodeValue::Dict(self.query.to_arguments().into_iter().collect()),
        );
        BencodeValue::Dict(dictionary.into_iter().collect())
    }

    pub fn try_from_bencoded(input: &BencodeValue) -> Result<Self, TryFromArgumentsError> {
        let dict = match input {
            BencodeValue::Dict(dict) => dict,
            _ => return Err("Invalid query - not a dictionary"),
        };

        let transaction_id = match dict.iter().find(|(key, _)| key == "t") {
            Some((_, BencodeValue::String(transaction_id))) => transaction_id.clone(),
            _ => return Err("Missing 't' field"),
        };
        let query_type = match dict.iter().find(|(key, _)| key == "q") {
            Some((_, BencodeValue::String(query_type))) => query_type,
            _ => return Err("Missing 'q' field"),
        };
        let arguments = match dict.iter().find(|(key, _)| key == "a") {
            Some((_, BencodeValue::Dict(arguments))) => arguments,
            _ => return Err("Missing 'a' field"),
        };

        let query = match query_type.as_str() {
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
    pub fn to_arguments(&self) -> HashMap<String, BencodeValue> {
        match self {
            QueryType::Ping(ping) => ping.to_arguments(),
            QueryType::FindNode(find_node) => find_node.to_arguments(),
            QueryType::GetPeers(get_peers) => get_peers.to_arguments(),
            QueryType::AnnouncePeer(announce_peer) => announce_peer.to_arguments(),
        }
    }

    pub fn get_query_type(&self) -> &str {
        match self {
            QueryType::Ping(_) => QUERY_TYPE_PING,
            QueryType::FindNode(_) => QUERY_TYPE_FIND_NODE,
            QueryType::GetPeers(_) => QUERY_TYPE_GET_PEERS,
            QueryType::AnnouncePeer(_) => QUERY_TYPE_ANNOUNCE_PEER,
        }
    }
}

impl<N: NodeId> ToArguments for Ping<N> {
    fn to_arguments(&self) -> HashMap<String, BencodeValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodeValue::String(self.id.to_string()));
        arguments
    }
}

impl<N: NodeId> ToArguments for FindNode<N> {
    fn to_arguments(&self) -> HashMap<String, BencodeValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodeValue::String(self.id.to_string()));
        arguments.insert(
            "target".to_string(),
            BencodeValue::String(self.target.to_string()),
        );
        arguments
    }
}

impl<N: NodeId> ToArguments for GetPeers<N> {
    fn to_arguments(&self) -> HashMap<String, BencodeValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodeValue::String(self.id.to_string()));
        arguments.insert(
            "info_hash".to_string(),
            BencodeValue::String(self.info_hash.to_string()),
        );
        arguments
    }
}

impl<N: NodeId> ToArguments for AnnouncePeer<N> {
    fn to_arguments(&self) -> HashMap<String, BencodeValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodeValue::String(self.id.to_string()));
        arguments.insert(
            "info_hash".to_string(),
            BencodeValue::String(self.info_hash.to_string()),
        );
        arguments.insert("port".to_string(), BencodeValue::Integer(self.port as i64));
        arguments.insert(
            "token".to_string(),
            BencodeValue::String(self.token.clone()),
        );
        arguments
    }
}

impl<N: NodeId> TryFromArguments for Ping<N> {
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key == "id")
            .ok_or("Missing 'id' field")?;
        if let BencodeValue::String(id) = id {
            Ok(Ping {
                id: N::from_str(id).or(Err("Invalid NodeId"))?,
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
            .find(|(key, _)| key == "id")
            .ok_or("Missing 'id' field")?;
        let (_, target) = arguments
            .iter()
            .find(|(key, _)| key == "target")
            .ok_or("Missing 'target' field")?;
        if let (BencodeValue::String(id), BencodeValue::String(target)) = (id, target) {
            Ok(FindNode {
                id: N::from_str(id).or(Err("Invalid NodeId"))?,
                target: N::from_str(target).or(Err("Invalid NodeId"))?,
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
            .find(|(key, _)| key == "id")
            .ok_or("Missing 'id' field")?;
        let (_, info_hash) = arguments
            .iter()
            .find(|(key, _)| key == "info_hash")
            .ok_or("Missing 'info_hash' field")?;
        if let (BencodeValue::String(id), BencodeValue::String(info_hash)) = (id, info_hash) {
            Ok(GetPeers {
                id: N::from_str(id).or(Err("Invalid NodeId"))?,
                info_hash: N::from_str(info_hash).or(Err("Invalid NodeId/InfoHash"))?,
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
            match key.as_str() {
                "id" => {
                    if let BencodeValue::String(id_) = value {
                        id = Some(N::from_str(&id_).or(Err("Invalid NodeId"))?);
                    } else {
                        return Err("Invalid 'id' field");
                    }
                }
                "info_hash" => {
                    if let BencodeValue::String(info_hash_) = value {
                        info_hash = Some(N::from_str(&info_hash_).or(Err("Invalid InfoHash"))?);
                    } else {
                        return Err("Invalid 'info_hash' field");
                    }
                }
                "port" => {
                    if let BencodeValue::Integer(port_) = value {
                        if *port_ < 0 || *port_ > u16::MAX as i64 {
                            return Err("Invalid 'port' field");
                        }
                        port = Some(*port_ as u16);
                    } else {
                        return Err("Invalid 'port' field");
                    }
                }
                "token" => {
                    if let BencodeValue::String(token_) = value {
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
    use std::str::FromStr;

    use super::super::tests::MockNodeId;
    use super::*;

    #[test]
    fn test_ping_query_to_bencoded() {
        let query = Query::new(
            "transaction_id".to_string(),
            QueryType::Ping(Ping {
                id: MockNodeId::from_str("25").unwrap(),
            }),
        );
        let mut bencoded = query.to_bencoded();
        let mut expected = BencodeValue::Dict(
            vec![
                (
                    "t".to_string(),
                    BencodeValue::String("transaction_id".to_string()),
                ),
                ("y".to_string(), BencodeValue::String("q".to_string())),
                ("q".to_string(), BencodeValue::String("ping".to_string())),
                (
                    "a".to_string(),
                    BencodeValue::Dict(
                        vec![("id".to_string(), BencodeValue::String("25".to_string()))]
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
