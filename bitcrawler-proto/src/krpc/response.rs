use std::collections::HashMap;

use crate::{
    bencoding::{BencodedDict, BencodedValue},
    kademlia::NodeId,
};

use super::{ToArguments, TryFromArguments, TryFromArgumentsError, query::QUERY_TYPE_PING};

/// Represents a response message in the KRPC protocol.
///
/// More information about the KRPC protocol can be found in the [specification](https://www.bittorrent.org/beps/bep_0005.html).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response<N: NodeId> {
    transaction_id: String,
    response: ResponseType<N>,
}

/// Represents a response type in the KRPC protocol.
///
/// Only 4 response types are supported: `ping`, `find_node`, `get_peers`, and `announce_peer`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ResponseType<N: NodeId> {
    /// Represents a `ping` query.
    Ping(Ping<N>),
    /*
    /// Represents a `find_node` query.
    FindNode(FindNode<N>),
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

impl<N: NodeId> Response<N> {
    pub fn new(transaction_id: String, response: ResponseType<N>) -> Self {
        Response {
            transaction_id,
            response,
        }
    }

    pub fn to_bencoded(&self) -> BencodedValue {
        let mut dictionary = HashMap::new();
        dictionary.insert(
            "t".to_string(),
            BencodedValue::String(self.transaction_id.clone()),
        );
        dictionary.insert("y".to_string(), BencodedValue::String("r".to_string()));
        dictionary.insert(
            "r".to_string(),
            BencodedValue::Dict(self.response.to_arguments().into_iter().collect()),
        );
        BencodedValue::Dict(dictionary.into_iter().collect())
    }

    pub fn try_from_ping_bencoded(bencoded: &BencodedValue) -> Result<Self, TryFromArgumentsError> {
        let bencoded = match bencoded {
            BencodedValue::Dict(bencoded) => bencoded,
            _ => return Err("Invalid response format"),
        };

        let (_, message_type) = bencoded
            .iter()
            .find(|(key, _)| key == "y")
            .ok_or("Missing 'y' field")?;
        if let BencodedValue::String(message_type) = message_type {
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
            BencodedValue::String(transaction_id) => transaction_id,
            _ => return Err("Invalid 't' field"),
        };

        let (_, response) = bencoded
            .iter()
            .find(|(key, _)| key == "r")
            .ok_or("Missing 'r' field")?;

        let response = match response {
            BencodedValue::Dict(response) => response,
            _ => return Err("Invalid 'r' field"),
        };

        let response_type = ResponseType::Ping(Ping::try_from_arguments(response)?);

        Ok(Response::new(transaction_id.to_string(), response_type))
    }
}

impl<N: NodeId> ResponseType<N> {
    pub fn to_arguments(&self) -> HashMap<String, BencodedValue> {
        match self {
            ResponseType::Ping(ping) => ping.to_arguments(),
        }
    }

    pub fn get_query_type(&self) -> &str {
        match self {
            ResponseType::Ping(_) => QUERY_TYPE_PING,
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::super::tests::MockNodeId;
    use super::*;

    #[test]
    fn test_ping_response_to_bencoded() {
        let response = Response::new(
            "123".to_string(),
            ResponseType::Ping(Ping {
                id: MockNodeId(123),
            }),
        );
        let mut bencoded = response.to_bencoded();
        let mut expected = BencodedValue::Dict(vec![
            ("t".to_string(), BencodedValue::String("123".to_string())),
            ("y".to_string(), BencodedValue::String("r".to_string())),
            (
                "r".to_string(),
                BencodedValue::Dict(vec![(
                    "id".to_string(),
                    BencodedValue::String("123".to_string()),
                )]),
            ),
        ]);
        bencoded.sort_keys();
        expected.sort_keys();
        assert_eq!(bencoded, expected);
    }

    #[test]
    fn test_ping_response_from_bencoded() {
        let bencoded = BencodedValue::Dict(vec![
            ("t".to_string(), BencodedValue::String("123".to_string())),
            ("y".to_string(), BencodedValue::String("r".to_string())),
            (
                "r".to_string(),
                BencodedValue::Dict(vec![(
                    "id".to_string(),
                    BencodedValue::String("123".to_string()),
                )]),
            ),
        ]);
        let response = Response::try_from_ping_bencoded(&bencoded).unwrap();
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
        let (_, bencoded) = crate::bencoding::decode(&bencoded_string).unwrap();
        let response = Response::try_from_ping_bencoded(&bencoded).unwrap();
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
}
