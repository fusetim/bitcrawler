use crate::{bencoding::BencodedValue, kademlia::NodeId};

mod query;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message<N: NodeId> {
    Query(query::Query<N>),
    //Response(query::Response),
    //Error(query::Error),
}

pub trait BencodedMessage {
    fn to_bencoded(&self) -> BencodedValue;
    fn from_bencoded(input: &BencodedValue) -> Self;
}

impl<N: NodeId> BencodedMessage for Message<N> {
    fn to_bencoded(&self) -> BencodedValue {
        match self {
            Message::Query(query) => query.to_bencoded(),
        }
    }

    fn from_bencoded(input: &BencodedValue) -> Self {
        unimplemented!()
    }
}