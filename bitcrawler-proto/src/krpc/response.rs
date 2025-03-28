use std::{collections::HashMap, ops::Add};

use std::str::FromStr;

use crate::{
    bencode::{BencodeDict, BencodeValue},
    kademlia::{Address, NodeId},
};

use super::query::QUERY_TYPE_FIND_NODE;
use super::{
    ToArguments, TryFromArguments, TryFromArgumentsError,
    node_info::{CompactNodeInfo, NodeInfo},
    query::QUERY_TYPE_PING,
};

/// Represents a response message in the KRPC protocol.
///
/// More information about the KRPC protocol can be found in the [specification](https://www.bittorrent.org/beps/bep_0005.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response<I: CompactNodeInfo> {
    transaction_id: String,
    response: ResponseType<I>,
}

/// Represents a response type in the KRPC protocol.
///
/// Only 4 response types are supported: `ping`, `find_node`, `get_peers`, and `announce_peer`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ResponseType<I: CompactNodeInfo> {
    /// Represents a `ping` query.
    Ping(Ping<I::NodeId>),
    /// Represents a `find_node` query.
    FindNode(FindNode<I>),
    /*
    /// Represents a `get_peers` query.
    GetPeers(GetPeers<N>),
    /// Represents an `announce_peer` query.
    AnnouncePeer(AnnouncePeer<N>),
    */
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Represents a `ping` response.
///
/// The `ping` query is used to test the liveness of a node.
/// See [Ping query](super::query::Ping) for more information.
pub struct Ping<N: NodeId> {
    id: N,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Represents a `find_node` response.
///
/// The `find_node` query is used to find the `k` nodes closest to a given `target`.
/// See [FindNode query](super::query::FindNode) for more information.
pub struct FindNode<I>
where
    I: CompactNodeInfo,
{
    id: I::NodeId,
    nodes: Vec<I>,
}

impl<I: CompactNodeInfo> Response<I> {
    pub fn new(transaction_id: String, response: ResponseType<I>) -> Self {
        Response {
            transaction_id,
            response,
        }
    }

    pub fn to_bencoded(&self) -> BencodeValue {
        let mut dictionary = HashMap::new();
        dictionary.insert(
            "t".to_string(),
            BencodeValue::String(self.transaction_id.clone()),
        );
        dictionary.insert("y".to_string(), BencodeValue::String("r".to_string()));
        dictionary.insert(
            "r".to_string(),
            BencodeValue::Dict(self.response.to_arguments().into_iter().collect()),
        );
        BencodeValue::Dict(dictionary.into_iter().collect())
    }

    pub fn try_from_ping_bencoded(bencoded: &BencodeValue) -> Result<Self, TryFromArgumentsError> {
        let bencoded = match bencoded {
            BencodeValue::Dict(bencoded) => bencoded,
            _ => return Err("Invalid response format"),
        };

        let (_, message_type) = bencoded
            .iter()
            .find(|(key, _)| key == "y")
            .ok_or("Missing 'y' field")?;
        if let BencodeValue::String(message_type) = message_type {
            if message_type != "r" {
                return Err("Invalid message type");
            }
        } else {
            return Err("Invalid 'y' field");
        }

        let (_, transaction_id) = bencoded
            .iter()
            .find(|(key, _)| key == "t")
            .ok_or("Missing 't' field")?;
        let transaction_id = match transaction_id {
            BencodeValue::String(transaction_id) => transaction_id,
            _ => return Err("Invalid 't' field"),
        };

        let (_, response) = bencoded
            .iter()
            .find(|(key, _)| key == "r")
            .ok_or("Missing 'r' field")?;

        let response = match response {
            BencodeValue::Dict(response) => response,
            _ => return Err("Invalid 'r' field"),
        };

        let response_type = ResponseType::Ping(Ping::try_from_arguments(response)?);

        Ok(Response::new(transaction_id.to_string(), response_type))
    }

    pub fn try_from_findpeer_bencoded(
        bencoded: &BencodeValue,
    ) -> Result<Self, TryFromArgumentsError> {
        let bencoded = match bencoded {
            BencodeValue::Dict(bencoded) => bencoded,
            _ => return Err("Invalid response format"),
        };

        let (_, message_type) = bencoded
            .iter()
            .find(|(key, _)| key == "y")
            .ok_or("Missing 'y' field")?;
        if let BencodeValue::String(message_type) = message_type {
            if message_type != "r" {
                return Err("Invalid message type");
            }
        } else {
            return Err("Invalid 'y' field");
        }

        let (_, transaction_id) = bencoded
            .iter()
            .find(|(key, _)| key == "t")
            .ok_or("Missing 't' field")?;
        let transaction_id = match transaction_id {
            BencodeValue::String(transaction_id) => transaction_id,
            _ => return Err("Invalid 't' field"),
        };

        let (_, response) = bencoded
            .iter()
            .find(|(key, _)| key == "r")
            .ok_or("Missing 'r' field")?;

        let response = match response {
            BencodeValue::Dict(response) => response,
            _ => return Err("Invalid 'r' field"),
        };

        let response_type = ResponseType::Ping(Ping::try_from_arguments(response)?);

        Ok(Response::new(transaction_id.to_string(), response_type))
    }
}

impl<I: CompactNodeInfo> ResponseType<I> {
    pub fn to_arguments(&self) -> HashMap<String, BencodeValue> {
        match self {
            ResponseType::Ping(ping) => ping.to_arguments(),
            ResponseType::FindNode(find_node) => find_node.to_arguments(),
        }
    }

    pub fn get_query_type(&self) -> &str {
        match self {
            ResponseType::Ping(_) => QUERY_TYPE_PING,
            ResponseType::FindNode(_) => QUERY_TYPE_FIND_NODE,
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

impl<I> ToArguments for FindNode<I>
where
    I: CompactNodeInfo,
{
    fn to_arguments(&self) -> HashMap<String, BencodeValue> {
        let mut arguments = HashMap::new();
        arguments.insert(
            "id".to_string(),
            BencodeValue::String(ToString::to_string(&self.id)),
        );
        let mut nodes = String::new();
        for node in &self.nodes {
            nodes.push_str(&node.write_compact_node_info());
        }
        arguments.insert("nodes".to_string(), BencodeValue::String(nodes));
        arguments
    }
}

impl<I> TryFromArguments for FindNode<I>
where
    I: CompactNodeInfo,
{
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key == "id")
            .ok_or("Missing 'id' field")?;
        let id = match id {
            BencodeValue::String(id) => id,
            _ => return Err("Invalid 'id' field"),
        };

        let (_, node_list) = arguments
            .iter()
            .find(|(key, _)| key == "nodes")
            .ok_or("Missing 'nodes' field")?;
        let node_list = match node_list {
            BencodeValue::String(nodes) => nodes,
            _ => return Err("Invalid 'nodes' field"),
        };

        let mut nodes = Vec::new();
        let mut i = 0;
        while i < node_list.len() {
            let (bytes_read, node) = match I::try_read_compact_node_info(&node_list[i..]) {
                Ok((bytes_read, node)) => (bytes_read, node),
                Err(_) => return Err("Invalid node info"),
            };
            nodes.push(node);
            i += bytes_read;
        }

        Ok(FindNode {
            id: I::NodeId::from_str(id).or(Err("Invalid NodeId"))?,
            nodes,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::super::tests::{MockNodeId, MockNodeInfo, MockAddress};
    use super::*;

    #[test]
    fn test_ping_response_to_bencoded() {
        let response = Response::<MockNodeInfo>::new(
            "123".to_string(),
            ResponseType::Ping(Ping {
                id: MockNodeId(123),
            }),
        );
        let mut bencoded = response.to_bencoded();
        let mut expected = BencodeValue::Dict(vec![
            ("t".to_string(), BencodeValue::String("123".to_string())),
            ("y".to_string(), BencodeValue::String("r".to_string())),
            (
                "r".to_string(),
                BencodeValue::Dict(vec![(
                    "id".to_string(),
                    BencodeValue::String("123".to_string()),
                )]),
            ),
        ]);
        bencoded.sort_keys();
        expected.sort_keys();
        assert_eq!(bencoded, expected);
    }

    #[test]
    fn test_ping_response_from_bencoded() {
        let bencoded = BencodeValue::Dict(vec![
            ("t".to_string(), BencodeValue::String("123".to_string())),
            ("y".to_string(), BencodeValue::String("r".to_string())),
            (
                "r".to_string(),
                BencodeValue::Dict(vec![(
                    "id".to_string(),
                    BencodeValue::String("123".to_string()),
                )]),
            ),
        ]);
        let response = Response::<MockNodeInfo>::try_from_ping_bencoded(&bencoded).unwrap();
        assert_eq!(
            response,
            Response::new(
                "123".to_string(),
                ResponseType::Ping(Ping {
                    id: MockNodeId(123),
                }),
            )
        );
    }

    #[test]
    fn test_ping_response_from_spec_bencoded() {
        let bencoded_string = "d1:rd2:id6:123456e1:t2:aa1:y1:re";
        let (_, bencoded) = crate::bencode::decode(&bencoded_string).unwrap();
        let response = Response::<MockNodeInfo>::try_from_ping_bencoded(&bencoded).unwrap();
        assert_eq!(
            response,
            Response::new(
                "aa".to_string(),
                ResponseType::Ping(Ping {
                    id: MockNodeId::from_str("123456").unwrap(),
                }),
            )
        );
    }

    #[test]
    fn test_findpeer_response_to_bencoded() {
        let response = Response::<MockNodeInfo>::new(
            "123".to_string(),
            ResponseType::FindNode(FindNode {
                id: MockNodeId(123),
                nodes: vec![
                    MockNodeInfo::new_with_address(
                        MockNodeId(123),
                        MockAddress {
                            ip: [1, 2, 3, 4],
                            port: 1234,
                        },
                    ),
                    MockNodeInfo::new_with_address(
                        MockNodeId(456),
                        MockAddress {
                            ip: [5, 6, 7, 8],
                            port: 5678,
                        },
                    ),
                ],
            }),
        );
        let mut bencoded = response.to_bencoded();
        let mut expected = BencodeValue::Dict(vec![
            ("t".to_string(), BencodeValue::String("123".to_string())),
            ("y".to_string(), BencodeValue::String("r".to_string())),
            (
                "r".to_string(),
                BencodeValue::Dict(vec![
                    (
                        "id".to_string(),
                        BencodeValue::String("123".to_string()),
                    ),
                    (
                        "nodes".to_string(),
                        BencodeValue::String(
                            "1234560102030412345600050607085678".to_string(),
                        ),
                    ),
                ]),
            ),
        ]);
        bencoded.sort_keys();
        expected.sort_keys();
        assert_eq!(bencoded, expected);
    }
}
