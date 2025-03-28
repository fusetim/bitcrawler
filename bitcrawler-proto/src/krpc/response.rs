use std::collections::HashMap;

use std::str::FromStr;

use crate::{
    bencode::{BencodeDict, BencodeString, BencodeValue},
    kademlia::NodeId,
};

use super::query::QUERY_TYPE_FIND_NODE;
use super::{
    ToArguments, TryFromArguments, TryFromArgumentsError, node_info::CompactNodeInfo,
    query::QUERY_TYPE_PING,
};

/// Represents a response message in the KRPC protocol.
///
/// More information about the KRPC protocol can be found in the [specification](https://www.bittorrent.org/beps/bep_0005.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response<I: CompactNodeInfo> {
    transaction_id: BencodeString,
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
    pub fn new(transaction_id: impl Into<BencodeString>, response: ResponseType<I>) -> Self {
        Response {
            transaction_id: transaction_id.into(),
            response,
        }
    }

    pub fn to_bencoded(&self) -> BencodeValue {
        let mut dictionary = HashMap::new();
        dictionary.insert(
            "t".into(),
            BencodeValue::ByteString(self.transaction_id.clone()),
        );
        dictionary.insert("y".into(), BencodeValue::ByteString("r".into()));
        dictionary.insert(
            "r".into(),
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
            .find(|(key, _)| key.as_ref() == b"y")
            .ok_or("Missing 'y' field")?;
        if let BencodeValue::ByteString(message_type) = message_type {
            if message_type.as_ref() != b"r" {
                return Err("Invalid message type");
            }
        } else {
            return Err("Invalid 'y' field");
        }

        let (_, transaction_id) = bencoded
            .iter()
            .find(|(key, _)| key.as_ref() == b"t")
            .ok_or("Missing 't' field")?;
        let transaction_id = match transaction_id {
            BencodeValue::ByteString(transaction_id) => transaction_id,
            _ => return Err("Invalid 't' field"),
        };

        let (_, response) = bencoded
            .iter()
            .find(|(key, _)| key.as_ref() == b"r")
            .ok_or("Missing 'r' field")?;

        let response = match response {
            BencodeValue::Dict(response) => response,
            _ => return Err("Invalid 'r' field"),
        };

        let response_type = ResponseType::Ping(Ping::try_from_arguments(response)?);

        Ok(Response::new(transaction_id.clone(), response_type))
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
            .find(|(key, _)| key.as_ref() == b"y")
            .ok_or("Missing 'y' field")?;
        if let BencodeValue::ByteString(message_type) = message_type {
            if message_type.as_ref() != b"r" {
                return Err("Invalid message type");
            }
        } else {
            return Err("Invalid 'y' field");
        }

        let (_, transaction_id) = bencoded
            .iter()
            .find(|(key, _)| key.as_ref() == b"t")
            .ok_or("Missing 't' field")?;
        let transaction_id = match transaction_id {
            BencodeValue::ByteString(transaction_id) => transaction_id,
            _ => return Err("Invalid 't' field"),
        };

        let (_, response) = bencoded
            .iter()
            .find(|(key, _)| key.as_ref() == b"r")
            .ok_or("Missing 'r' field")?;

        let response = match response {
            BencodeValue::Dict(response) => response,
            _ => return Err("Invalid 'r' field"),
        };

        let response_type = ResponseType::Ping(Ping::try_from_arguments(response)?);

        Ok(Response::new(transaction_id.clone(), response_type))
    }
}

impl<I: CompactNodeInfo> ResponseType<I> {
    pub fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        match self {
            ResponseType::Ping(ping) => ping.to_arguments(),
            ResponseType::FindNode(find_node) => find_node.to_arguments(),
        }
    }

    pub fn get_query_type(&self) -> &[u8] {
        match self {
            ResponseType::Ping(_) => QUERY_TYPE_PING,
            ResponseType::FindNode(_) => QUERY_TYPE_FIND_NODE,
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

impl<I> ToArguments for FindNode<I>
where
    I: CompactNodeInfo,
{
    fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        let mut arguments = HashMap::new();
        let id: Vec<u8> = self.id.clone().into();
        arguments.insert("id".into(), BencodeValue::ByteString(id.into()));
        let mut nodes = Vec::new();
        for node in &self.nodes {
            nodes.extend(node.write_compact_node_info());
        }
        arguments.insert("nodes".into(), BencodeValue::ByteString(nodes.into()));
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
            .find(|(key, _)| key.as_ref() == b"id")
            .ok_or("Missing 'id' field")?;
        let id = match id {
            BencodeValue::ByteString(id) => id,
            _ => return Err("Invalid 'id' field"),
        };

        let (_, node_list) = arguments
            .iter()
            .find(|(key, _)| key.as_ref() == b"nodes")
            .ok_or("Missing 'nodes' field")?;
        let node_list = match node_list {
            BencodeValue::ByteString(nodes) => nodes,
            _ => return Err("Invalid 'nodes' field"),
        };

        let mut nodes = Vec::new();
        let mut i = 0;
        while i < node_list.as_ref().len() {
            let (bytes_read, node) = match I::try_read_compact_node_info(&node_list.as_ref()[i..]) {
                Ok((bytes_read, node)) => (bytes_read, node),
                Err(_) => return Err("Invalid node info"),
            };
            nodes.push(node);
            i += bytes_read;
        }

        Ok(FindNode {
            id: I::NodeId::try_from(id.as_ref()).or(Err("Invalid NodeId"))?,
            nodes,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::super::tests::{MockAddress, MockNodeId, MockNodeInfo};
    use super::*;

    #[test]
    fn test_ping_response_to_bencoded() {
        let response = Response::<MockNodeInfo>::new(
            "123",
            ResponseType::Ping(Ping {
                id: MockNodeId(123),
            }),
        );
        let mut bencoded = response.to_bencoded();
        let mut expected = BencodeValue::Dict(vec![
            ("t".into(), BencodeValue::ByteString("123".into())),
            ("y".into(), BencodeValue::ByteString("r".into())),
            (
                "r".into(),
                BencodeValue::Dict(vec![("id".into(), BencodeValue::ByteString(vec![0, 0, 0, 0, 0, 0, 0, 123].into()))]),
            ),
        ]);
        bencoded.sort_keys();
        expected.sort_keys();
        assert_eq!(bencoded, expected);
    }

    #[test]
    fn test_ping_response_from_bencoded() {
        let bencoded = BencodeValue::Dict(vec![
            ("t".into(), BencodeValue::ByteString("123".into())),
            ("y".into(), BencodeValue::ByteString("r".into())),
            (
                "r".into(),
                BencodeValue::Dict(vec![(
                    "id".into(),
                    BencodeValue::ByteString("12345678".into()),
                )]),
            ),
        ]);
        let response = Response::<MockNodeInfo>::try_from_ping_bencoded(&bencoded).unwrap();
        assert_eq!(
            response,
            Response::new(
                "123".to_string(),
                ResponseType::Ping(Ping {
                    id: MockNodeId::try_from(b"12345678".as_ref()).unwrap(),
                }),
            )
        );
    }

    #[test]
    fn test_ping_response_from_spec_bencoded() {
        let bencoded_string = "d1:rd2:id8:12345678e1:t2:aa1:y1:re";
        let (_, bencoded) = crate::bencode::decode(&bencoded_string).unwrap();
        let response = Response::<MockNodeInfo>::try_from_ping_bencoded(&bencoded).unwrap();
        assert_eq!(
            response,
            Response::new(
                "aa".to_string(),
                ResponseType::Ping(Ping {
                    id: MockNodeId::try_from(b"12345678".as_ref()).unwrap(),
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
                    MockNodeInfo {
                        node_id: MockNodeId(128),
                        ip: [1, 2, 3, 4],
                        port: 1234,
                    },
                    MockNodeInfo {
                        node_id: MockNodeId(129),
                        ip: [5, 6, 7, 8],
                        port: 5678,
                    },
                ],
            }),
        );
        let mut bencoded = response.to_bencoded();
        let mut expected = BencodeValue::Dict(vec![
            ("t".into(), BencodeValue::ByteString("123".into())),
            ("y".into(), BencodeValue::ByteString("r".into())),
            (
                "r".into(),
                BencodeValue::Dict(vec![
                    (
                        "id".into(),
                        BencodeValue::ByteString(vec![0, 0, 0, 0, 0, 0, 0, 123].into()),
                    ),
                    (
                        "nodes".into(),
                        BencodeValue::ByteString(
                            vec![
                                0, 0, 0, 0, 0, 0, 0, 128, 1, 2, 3, 4, 4, 210, 0, 0, 0, 0, 0, 0, 0,
                                129, 5, 6, 7, 8, 22, 46,
                            ]
                            .into(),
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
