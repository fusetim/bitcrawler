mod error;
pub mod node_info;
pub mod query;
pub mod response;

use std::collections::HashMap;

use crate::{
    bencoding::{BencodeDict, BencodeValue},
    kademlia::NodeId,
};
pub use error::*;
pub use query::{Query, QueryType};
pub use response::{Response, ResponseType};

/// Represents a KRPC message that can be either a query, a response, or an error.
///
/// # Variants
///
/// - `Query`: Represents a query message containing a `Query` object.
///
/// # Type Parameters
///
/// - `N`: A type that implements the `NodeId` trait, representing the identifier of a node in the network.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message<N: NodeId> {
    Query(query::Query<N>),
    Error(error::ErrorMessage),
}

/// A trait for encoding and decoding messages using the Bencoding format.
///
/// # Required Methods
///
/// - `to_bencoded`: Converts the implementing type into a `BencodedValue`.
/// - `from_bencoded`: Constructs an instance of the implementing type from a `BencodedValue`.
pub trait BencodedMessage {
    /// Converts the message into a `BencodedValue`.
    ///
    /// # Returns
    ///
    /// A `BencodedValue` representation of the message.
    fn to_bencoded(&self) -> BencodeValue;

    /// Constructs an instance of the message from a `BencodedValue`.
    ///
    /// # Parameters
    ///
    /// - `input`: The `BencodedValue` to construct the message from.
    ///
    /// # Returns
    ///
    /// A new instance of the message if the `BencodedValue` is valid, otherwise an error message.
    fn try_from_bencoded(input: &BencodeValue) -> Result<Self, &'static str>
    where
        Self: Sized;
}

impl<N: NodeId> BencodedMessage for Message<N> {
    fn to_bencoded(&self) -> BencodeValue {
        match self {
            Message::Query(query) => query.to_bencoded(),
            Message::Error(error) => error.to_bencoded(),
        }
    }

    fn try_from_bencoded(input: &BencodeValue) -> Result<Self, &'static str> {
        let dict = match input {
            BencodeValue::Dict(dict) => dict,
            _ => return Err("Invalid message format"),
        };

        let y = match dict.iter().find(|(key, _)| key == "y") {
            Some((_, BencodeValue::String(y))) => y,
            _ => return Err("Missing 'y' key"),
        };

        match y.as_str() {
            "q" => query::Query::try_from_bencoded(input).map(Message::Query),
            //"r" => response::Response::try_from_bencoded(input).map(Message::Query),
            "e" => error::ErrorMessage::try_from_bencoded(input).map(Message::Error),
            _ => Err("Invalid message type"),
        }
    }
}

/// A trait for converting a type into a collection of key-value pairs, called arguments in the KRPC protocol.
pub trait ToArguments {
    /// Converts the implementing type into a collection of key-value pairs.
    fn to_arguments(&self) -> HashMap<String, BencodeValue>;
}

/// A trait for converting a collection of key-value pairs, called arguments in the KRPC protocol, into a type.
pub type TryFromArgumentsError = &'static str;
pub trait TryFromArguments {
    /// Constructs an instance of the implementing type from a collection of key-value pairs.
    fn try_from_arguments(arguments: &BencodeDict) -> Result<Self, TryFromArgumentsError>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::{node_info::CompactNodeInfo, *};
    use std::str::FromStr;

    use crate::kademlia::Xorable;

    #[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
    pub struct MockNodeId(pub u64);

    pub type MockNodeInfo = node_info::BittorrentNodeInfoV4<MockNodeId>;
    pub struct MockAddress {
        pub ip: [u8; 4],
        pub port: u16,
    }

    impl NodeId for MockNodeId {}

    impl node_info::NodeInfo for MockNodeInfo {
        type NodeId = MockNodeId;
        type Address = MockAddress;

        fn get_node_id(&self) -> &Self::NodeId {
            &self.node_id
        }

        fn to_address(&self) -> Self::Address {
            MockAddress {
                ip: self.ip,
                port: self.port,
            }
        }

        fn new_with_address(node_id: Self::NodeId, address: Self::Address) -> Self {
            MockNodeInfo {
                node_id,
                ip: address.ip,
                port: address.port,
            }
        }
    }

    impl CompactNodeInfo for MockNodeInfo {
        type Error = &'static str;

        fn try_read_compact_node_info(data: &str) -> Result<(usize, Self), Self::Error> {
            let mut iter = data.bytes();
            let node_id = MockNodeId(
                iter.by_ref()
                    .take(8)
                    .fold(0u64, |acc, x| (acc << 8) | x as u64),
            );
            let ip = [
                iter.next().ok_or("Invalid compact node info")?,
                iter.next().ok_or("Invalid compact node info")?,
                iter.next().ok_or("Invalid compact node info")?,
                iter.next().ok_or("Invalid compact node info")?,
            ];
            let port = ((iter.next().ok_or("Invalid compact node info")? as u16) << 8)
                | iter.next().ok_or("Invalid compact node info")? as u16;
            Ok((14, MockNodeInfo { node_id, ip, port }))
        }

        fn write_compact_node_info(&self) -> std::borrow::Cow<str> {
            let mut data = Vec::with_capacity(6);
            data.extend_from_slice(&self.node_id.0.to_be_bytes());
            data.extend_from_slice(&self.ip);
            data.extend_from_slice(&self.port.to_be_bytes());
            String::from_utf8(data).unwrap().into()
        }
    }

    impl FromStr for MockNodeId {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.parse::<u64>() {
                Ok(id) => Ok(MockNodeId(id)),
                Err(_) => Err("Invalid NodeId"),
            }
        }
    }

    impl ToString for MockNodeId {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }

    impl Xorable for MockNodeId {
        fn cmp_distance(&self, other: &Self) -> std::cmp::Ordering {
            self.0.cmp(&other.0)
        }

        fn bucket_index(&self, other: &Self) -> usize {
            let x = self.0 ^ other.0;
            let mut count = 0;
            while (x >> count) > 1 {
                count += 1;
            }
            return count;
        }
    }
}
