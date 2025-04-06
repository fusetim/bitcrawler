use std::collections::HashMap;

use crate::{
    bencode::{BencodeDict, BencodeString, BencodeValue},
    kademlia::NodeId,
};

use super::{peer_info::CompactPeerInfo, query::{QUERY_TYPE_FIND_NODE, QUERY_TYPE_GET_PEERS}};
use super::{
    ToArguments, TryFromArguments, TryFromArgumentsError, node_info::CompactNodeInfo,
    query::QUERY_TYPE_PING,
};

/// Represents a response message in the KRPC protocol.
///
/// More information about the KRPC protocol can be found in the [specification](https://www.bittorrent.org/beps/bep_0005.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response<I: CompactNodeInfo, P: CompactPeerInfo> {
    transaction_id: BencodeString,
    response: ResponseType<I, P>,
}

/// Represents a response type in the KRPC protocol.
///
/// Only 4 response types are supported: `ping`, `find_node`, `get_peers`, and `announce_peer`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ResponseType<I: CompactNodeInfo, P: CompactPeerInfo> {
    /// Represents a `ping` query.
    Ping(Ping<I::NodeId>),
    /// Represents a `find_node` query.
    FindNode(FindNode<I>),
    /// Represents a `get_peers` query.
    GetPeers(GetPeers<I, P>),
    /*
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

#[derive(Debug, PartialEq, Eq, Clone)]
/// Represents a `get_peers` response.
/// 
/// The `get_peers` query is used to find the `k` nodes closest to a given `target` info_hash.
/// See [GetPeers query](super::query::GetPeers) for more information.
/// The `peers` field contains either a list of compact peer info or a list of compact nodes to contact.
pub struct GetPeers<I: CompactNodeInfo, P: CompactPeerInfo> {
    id: I::NodeId,
    // (Optional) token used to broadcast an announce_peer query
    // to the tracker. The token is used to prevent abuse of the tracker.
    token: Option<BencodeString>,
    nodes: Vec<I>,
    peers: Vec<P>,
}

impl<I: CompactNodeInfo, P: CompactPeerInfo> Response<I, P> {
    pub fn new(transaction_id: impl Into<BencodeString>, response: ResponseType<I, P>) -> Self {
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

    fn try_from_bencoded_internal(bencoded: &BencodeValue) -> Result<(BencodeString, Vec<(BencodeString, BencodeValue)>), TryFromArgumentsError> {
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

        match response {
            BencodeValue::Dict(response) => Ok((transaction_id.clone(), response.clone())),
            _ => return Err("Invalid 'r' field"),
        }
    }

    pub fn try_guess_type_from_bencoded(
        bencoded: &BencodeValue,
    ) -> Result<(&'static [u8], BencodeString), TryFromArgumentsError> {
        let (transaction_id, response) = Self::try_from_bencoded_internal(bencoded)?;
        
        let (mut has_values_field,mut has_token_field, mut has_nodes_field) = (false, false, false);
        for (key, value) in response {
            match key.as_ref() {
                b"values" => has_values_field = true,
                b"token" => has_token_field = true,
                b"nodes" => has_nodes_field = true,
                _ => {}
            }
        }

        match (has_values_field, has_token_field, has_nodes_field) {
            (true, _, _) => Ok((QUERY_TYPE_GET_PEERS, transaction_id)),
            (_, true, _) => Ok((QUERY_TYPE_GET_PEERS, transaction_id)),
            (_, _, true) => Ok((QUERY_TYPE_FIND_NODE, transaction_id)),
            (false, false, false) => Ok((QUERY_TYPE_PING, transaction_id)),
        }
    }

    pub fn try_from_ping_bencoded(bencoded: &BencodeValue) -> Result<Self, TryFromArgumentsError> {
        match Self::try_from_bencoded_internal(bencoded) {
            Ok((transaction_id, response)) => {
                let response_type = ResponseType::Ping(Ping::try_from_arguments(&response)?);
                Ok(Response::new(transaction_id, response_type))
            }
            Err(e) => Err(e),
        }
    }

    pub fn try_from_findpeer_bencoded(
        bencoded: &BencodeValue,
    ) -> Result<Self, TryFromArgumentsError> {
        match Self::try_from_bencoded_internal(bencoded) {
            Ok((transaction_id, response)) => {
                let response_type = ResponseType::FindNode(FindNode::try_from_arguments(&response)?);
                Ok(Response::new(transaction_id, response_type))
            }
            Err(e) => Err(e),
        }
    }

    pub fn try_from_getpeers_bencoded(
        bencoded: &BencodeValue,
    ) -> Result<Self, TryFromArgumentsError> {
        match Self::try_from_bencoded_internal(bencoded) {
            Ok((transaction_id, response)) => {
                let response_type = ResponseType::GetPeers(GetPeers::try_from_arguments(&response)?);
                Ok(Response::new(transaction_id, response_type))
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_transaction_id(&self) -> &BencodeString {
        &self.transaction_id
    }

    pub fn get_response_type(&self) -> &ResponseType<I, P> {
        &self.response
    }
}

impl<I: CompactNodeInfo, P: CompactPeerInfo> ResponseType<I, P> {
    pub fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        match self {
            ResponseType::Ping(ping) => ping.to_arguments(),
            ResponseType::FindNode(find_node) => find_node.to_arguments(),
            ResponseType::GetPeers(get_peers) => get_peers.to_arguments(),
        }
    }

    pub fn get_query_type(&self) -> &[u8] {
        match self {
            ResponseType::Ping(_) => QUERY_TYPE_PING,
            ResponseType::FindNode(_) => QUERY_TYPE_FIND_NODE,
            ResponseType::GetPeers(_) => QUERY_TYPE_FIND_NODE,
        }
    }
}

impl<N: NodeId> Ping<N> {
    pub fn get_id(&self) -> &N {
        &self.id
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

impl<I: CompactNodeInfo, P: CompactPeerInfo> GetPeers<I, P> {
    pub fn get_id(&self) -> &I::NodeId {
        &self.id
    }

    pub fn get_token(&self) -> &Option<BencodeString> {
        &self.token
    }

    pub fn get_nodes(&self) -> &[I] {
        &self.nodes
    }

    pub fn get_peers(&self) -> &[P] {
        &self.peers
    }
}

impl<I: CompactNodeInfo, P: CompactPeerInfo> ToArguments for GetPeers<I, P> {
    fn to_arguments(&self) -> HashMap<BencodeString, BencodeValue> {
        let mut arguments = HashMap::new();
        let id: Vec<u8> = self.id.clone().into();
        arguments.insert("id".into(), BencodeValue::ByteString(id.into()));
        if let Some(token) = &self.token {
            arguments.insert("token".into(), BencodeValue::ByteString(token.clone()));
        }
        let mut nodes = Vec::new();
        for node in &self.nodes {
            nodes.extend(node.write_compact_node_info());
        }
        arguments.insert("nodes".into(), BencodeValue::ByteString(nodes.into()));
        let mut peers = Vec::new();
        for peer in &self.peers {
            peers.extend(peer.write_compact_peer_info());
        }
        // NOTE: The peers field is actually named "values" in the KRPC protocol
        // but we use "peers" for clarity.
        arguments.insert("values".into(), BencodeValue::ByteString(peers.into()));
        arguments
    }
}

impl<I: CompactNodeInfo, P: CompactPeerInfo> TryFromArguments for GetPeers<I, P> {
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError> {
        let (_, id) = arguments
            .iter()
            .find(|(key, _)| key.as_ref() == b"id")
            .ok_or("Missing 'id' field")?;
        let id = match id {
            BencodeValue::ByteString(id) => id,
            _ => return Err("Invalid 'id' field"),
        };

        // The token field is optional, so we need to check if it exists
        let token = {
            match arguments.iter().find(|(key, _)| key.as_ref() == b"token") {
                Some((_, token_bencoded)) => match token_bencoded {
                    BencodeValue::ByteString(token_string) => Some(token_string.clone()),
                    _ => return Err("Invalid 'token' field"),
                },
                None => None,
            }
        };

        // The nodes field is optional, so we need to check if it exists
        let node_list = {
            match arguments.iter().find(|(key, _)| key.as_ref() == b"nodes") {
                Some((_, node_bencoded)) => match node_bencoded {
                    BencodeValue::ByteString(node_string) => {
                        // Decode the nodes into a vector of node info
                        let mut nodes = Vec::new();
                        let mut i = 0;
                        while i < node_string.as_ref().len() {
                            match I::try_read_compact_node_info(&node_string.as_ref()[i..]) {
                                Ok((bytes_read, node)) => {
                                    nodes.push(node);
                                    i += bytes_read;
                                },
                                Err(_) => return Err("Invalid node info"),
                            }
                        }
                        nodes
                    },
                    _ => return Err("Invalid 'nodes' field"),
                },
                None => Vec::new(),
            }
        };

        // The peers field is optional, so we need to check if it exists
        let peer_list = {
            match arguments.iter().find(|(key, _)| key.as_ref() == b"values") {
                Some((_, peer_bencoded)) => match peer_bencoded {
                    BencodeValue::List(peer_infos) => {
                        // Decode the peers into a vector of peer info
                        let mut peers = Vec::new();
                        for peer_info in peer_infos {
                            match peer_info {
                                BencodeValue::ByteString(peer_info) => {
                                    peers.push(P::try_read_compact_peer_info(peer_info.as_ref())
                                        .map(|(è, peer)| peer)
                                        .map_err(|_| "Invalid peer info")?);
                                },
                                _ => return Err("Invalid peer info"),
                            }
                        }
                        peers
                    },
                    _ => return Err("Invalid 'peers' field"),
                },
                None => Vec::new(),
            }
        };

        Ok(GetPeers {
            id: I::NodeId::try_from(id.as_ref()).or(Err("Invalid NodeId"))?,
            token,
            nodes: node_list,
            peers: peer_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::krpc::tests::MockAddress;

    use super::super::tests::{MockNodeId, MockNodeInfo};
    use super::*;

    #[test]
    fn test_ping_response_to_bencoded() {
        let response = Response::<MockNodeInfo, MockAddress>::new(
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
                BencodeValue::Dict(vec![(
                    "id".into(),
                    BencodeValue::ByteString(vec![0, 0, 0, 0, 0, 0, 0, 123].into()),
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
        let response = Response::<MockNodeInfo, MockAddress>::try_from_ping_bencoded(&bencoded).unwrap();
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
        let response = Response::<MockNodeInfo, MockAddress>::try_from_ping_bencoded(&bencoded).unwrap();
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
        let response = Response::<MockNodeInfo, MockAddress>::new(
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

    #[test]
    fn test_get_peers_from_bencoded() {
        let bencoded = BencodeValue::Dict(vec![
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
                        "token".into(),
                        BencodeValue::ByteString(vec![0,1,2,3].into()),
                    ),
                    (
                        "nodes".into(),
                        BencodeValue::ByteString( vec![
                            /* Node 1 */
                            0, 0, 0, 0, 0, 0, 0, 128, 1, 2, 3, 4, 4, 210, 
                            /* Node 2 */
                            0, 0, 0, 0, 0, 0, 0, 129, 5, 6, 7, 8, 22, 46,
                        ]
                        .into()),
                    ),
                    (
                        "values".into(),
                        BencodeValue::List(vec![
                            BencodeValue::ByteString(vec![1, 2, 3, 4, 4, 210].into()),
                            BencodeValue::ByteString(vec![5, 6, 7, 8, 22, 46].into()),
                        ]),
                    ),
                ]),
            ),
        ]);
        let response = Response::<MockNodeInfo, MockAddress>::try_from_getpeers_bencoded(&bencoded).unwrap();
        assert_eq!(
            response,
            Response::new(
                "123".to_string(),
                ResponseType::GetPeers(GetPeers {
                    id: MockNodeId(123),
                    token: Some([0, 1, 2, 3].as_ref().into()),
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
                    peers: vec![
                        MockAddress {
                            ip: [1, 2, 3, 4],
                            port: 1234,
                        },
                        MockAddress {
                            ip: [5, 6, 7, 8],
                            port: 5678,
                        },
                    ],
                }),
            )
        );
    }
}
