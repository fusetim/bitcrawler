mod query;

pub use query::*;
use crate::{bencoding::BencodedValue, kademlia::NodeId};

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
    fn to_bencoded(&self) -> BencodedValue;

    /// Constructs a message from a `BencodedValue`.
    ///
    /// # Parameters
    ///
    /// - `input`: A reference to a `BencodedValue` to decode.
    ///
    /// # Returns
    ///
    /// An instance of the implementing type.
    fn from_bencoded(input: &BencodedValue) -> Self;
}

impl<N: NodeId> BencodedMessage for Message<N> {
    /// Converts the `Message` into a `BencodedValue`.
    ///
    /// # Returns
    ///
    /// A `BencodedValue` representation of the `Message`.
    fn to_bencoded(&self) -> BencodedValue {
        match self {
            Message::Query(query) => query.to_bencoded(),
        }
    }

    /// Constructs a `Message` from a `BencodedValue`.
    ///
    /// # Parameters
    ///
    /// - `input`: A reference to a `BencodedValue` to decode.
    ///
    /// # Panics
    ///
    /// This method is not yet implemented and will panic if called.
    fn from_bencoded(input: &BencodedValue) -> Self {
        unimplemented!()
    }
}