use std::collections::HashMap;

use crate::{
    bencoding::{BencodedDict, BencodedValue},
    kademlia::{Address, NodeId},
};

pub const QUERY_TYPE_PING: &str = "ping";
pub const QUERY_TYPE_FIND_NODE: &str = "find_node";
pub const QUERY_TYPE_GET_PEERS: &str = "get_peers";
pub const QUERY_TYPE_ANNOUNCE_PEER: &str = "announce_peer";

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct Query<N: NodeId> {
    transaction_id: String,
    query: QueryType<N>,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub enum QueryType<N: NodeId> {
    Ping(Ping<N>),
    FindNode(FindNode<N>),
    GetPeers(GetPeers<N>),
    AnnouncePeer(AnnouncePeer<N>),
}

pub trait ToArguments {
    fn to_arguments(&self) -> HashMap<String, BencodedValue>;
}

pub type TryFromArgumentsError = &'static str;
pub trait TryFromArguments {
    fn try_from_arguments(arguments: &BencodedDict) -> Result<Self, TryFromArgumentsError>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct Ping<N: NodeId> {
    id: N,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct FindNode<N: NodeId> {
    id: N,
    target: N,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub struct GetPeers<N: NodeId> {
    id: N,
    info_hash: N,
}

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

    pub fn to_bencoded(&self) -> BencodedValue {
        let mut dictionary = HashMap::new();
        dictionary.insert(
            "t".to_string(),
            BencodedValue::String(self.transaction_id.clone()),
        );
        dictionary.insert("y".to_string(), BencodedValue::String("q".to_string()));
        dictionary.insert(
            "q".to_string(),
            BencodedValue::String(self.query.get_query_type().to_string()),
        );
        dictionary.insert(
            "a".to_string(),
            BencodedValue::Dict(self.query.to_arguments().into_iter().collect()),
        );
        BencodedValue::Dict(dictionary.into_iter().collect())
    }

    pub fn try_from_bencoded(input: &BencodedValue) -> Result<Self, TryFromArgumentsError> {
        let dict = match input {
            BencodedValue::Dict(dict) => dict,
            _ => return Err("Invalid query - not a dictionary"),
        };

        let transaction_id = match dict.iter().find(|(key, _)| key == "t") {
            Some((_, BencodedValue::String(transaction_id))) => transaction_id.clone(),
            _ => return Err("Missing 't' field"),
        };
        let query_type = match dict.iter().find(|(key, _)| key == "q") {
            Some((_, BencodedValue::String(query_type))) => query_type,
            _ => return Err("Missing 'q' field"),
        };
        let arguments = match dict.iter().find(|(key, _)| key == "a") {
            Some((_, BencodedValue::Dict(arguments))) => arguments,
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
    pub fn to_arguments(&self) -> HashMap<String, BencodedValue> {
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
    fn to_arguments(&self) -> HashMap<String, BencodedValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodedValue::String(self.id.to_string()));
        arguments
    }
}

impl<N: NodeId> ToArguments for FindNode<N> {
    fn to_arguments(&self) -> HashMap<String, BencodedValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodedValue::String(self.id.to_string()));
        arguments.insert(
            "target".to_string(),
            BencodedValue::String(self.target.to_string()),
        );
        arguments
    }
}

impl<N: NodeId> ToArguments for GetPeers<N> {
    fn to_arguments(&self) -> HashMap<String, BencodedValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodedValue::String(self.id.to_string()));
        arguments.insert(
            "info_hash".to_string(),
            BencodedValue::String(self.info_hash.to_string()),
        );
        arguments
    }
}

impl<N: NodeId> ToArguments for AnnouncePeer<N> {
    fn to_arguments(&self) -> HashMap<String, BencodedValue> {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), BencodedValue::String(self.id.to_string()));
        arguments.insert(
            "info_hash".to_string(),
            BencodedValue::String(self.info_hash.to_string()),
        );
        arguments.insert("port".to_string(), BencodedValue::Integer(self.port as i64));
        arguments.insert(
            "token".to_string(),
            BencodedValue::String(self.token.clone()),
        );
        arguments
    }
}

impl<N: NodeId> TryFromArguments for Ping<N> {
    fn try_from_arguments(arguments: &BencodedDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key == "id")
            .ok_or("Missing 'id' field")?;
        if let BencodedValue::String(id) = id {
            Ok(Ping {
                id: N::from_str(id).or(Err("Invalid NodeId"))?,
            })
        } else {
            Err("Invalid 'id' field")
        }
    }
}

impl<N: NodeId> TryFromArguments for FindNode<N> {
    fn try_from_arguments(arguments: &BencodedDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key == "id")
            .ok_or("Missing 'id' field")?;
        let (_, target) = arguments
            .iter()
            .find(|(key, _)| key == "target")
            .ok_or("Missing 'target' field")?;
        if let (BencodedValue::String(id), BencodedValue::String(target)) = (id, target) {
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
    fn try_from_arguments(arguments: &BencodedDict) -> Result<Self, TryFromArgumentsError>
    {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key == "id")
            .ok_or("Missing 'id' field")?;
        let (_, info_hash) = arguments
            .iter()
            .find(|(key, _)| key == "info_hash")
            .ok_or("Missing 'info_hash' field")?;
        if let (BencodedValue::String(id), BencodedValue::String(info_hash)) = (id, info_hash) {
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
    fn try_from_arguments(arguments: &BencodedDict) -> Result<Self, TryFromArgumentsError>
    {
        let (mut id, mut info_hash, mut port, mut token) = (None, None, None, None);
        for (key, value) in arguments {
            match key.as_str() {
                "id" => {
                    if let BencodedValue::String(id_) = value {
                        id = Some(N::from_str(&id_).or(Err("Invalid NodeId"))?);
                    } else {
                        return Err("Invalid 'id' field");
                    }
                }
                "info_hash" => {
                    if let BencodedValue::String(info_hash_) = value {
                        info_hash = Some(N::from_str(&info_hash_).or(Err("Invalid InfoHash"))?);
                    } else {
                        return Err("Invalid 'info_hash' field");
                    }
                }
                "port" => {
                    if let BencodedValue::Integer(port_) = value {
                        if *port_ < 0 || *port_ > u16::MAX as i64 {
                            return Err("Invalid 'port' field");
                        }
                        port = Some(*port_ as u16);
                    } else {
                        return Err("Invalid 'port' field");
                    }
                }
                "token" => {
                    if let BencodedValue::String(token_) = value {
                        token = Some(token_.clone());
                    } else {
                        return Err("Invalid 'token' field");
                    }
                }
                _ => {/* Ignore */},
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