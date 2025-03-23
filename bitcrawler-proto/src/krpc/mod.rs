mod query;
mod response;
mod error;

use crate::{bencoding::BencodedValue, kademlia::NodeId};
pub use query::*;
pub use response::*;
pub use error::*;

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
    fn to_bencoded(&self) -> BencodedValue;

    /// Constructs an instance of the message from a `BencodedValue`.
    /// 
    /// # Parameters
    /// 
    /// - `input`: The `BencodedValue` to construct the message from.
    /// 
    /// # Returns
    /// 
    /// A new instance of the message if the `BencodedValue` is valid, otherwise an error message.
    fn try_from_bencoded(input: &BencodedValue) -> Result<Self, String> where Self: Sized;
}

impl<N: NodeId> BencodedMessage for Message<N> {
    fn to_bencoded(&self) -> BencodedValue {
        match self {
            Message::Query(query) => query.to_bencoded(),
            Message::Error(error) => error.to_bencoded(),
        }
    }

    fn try_from_bencoded(input: &BencodedValue) -> Result<Self, String> {
        todo!();
    }
}
