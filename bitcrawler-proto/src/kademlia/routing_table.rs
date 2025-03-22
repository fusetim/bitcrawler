use std::cmp::{Ordering, min};
use std::fmt::Debug;
use std::str::FromStr;

/// An `Address` is a type that represents a network address that can be used to
/// contact a node in a distributed system. This trait is intended to be
/// implemented by types that represent network addresses, such as IP addresses
/// or URLs.
pub trait Address: PartialEq + Debug {}

/// A `NodeId` is a type that represents a unique identifier for a node in a
/// distributed system. This trait is intended to be implemented by types that
/// represent node identifiers, such as public keys or hashes.
pub trait NodeId: PartialEq + Debug + Eq + Xorable + PartialOrd + Ord + Clone + ToString + FromStr {}

/// A trait that defines operations for comparing and calculating distances
/// between elements in a XOR-based metric space, commonly used in distributed
/// systems like Kademlia.
///
/// # Required Methods
///
/// - `cmp_distance`: Compares the distance between `self` and `other` and
///   returns an `Ordering` indicating their relative distances.
/// - `bucket_index`: Calculates the bucket index for `other` relative to `self`,
///   which is typically used to determine the appropriate bucket in a routing
///   table (number of leading bits that are identical).
///
/// This trait is intended to be implemented by types that represent keys or
/// identifiers in a distributed hash table (DHT) or similar systems.
pub trait Xorable {
    /// Compares the distance between `self` and `other` and returns an `Ordering`
    /// indicating their relative distances.
    fn cmp_distance(&self, other: &Self) -> Ordering;

    /// Calculates the bucket index for `other` relative to `self`, which is
    /// typically used to determine the appropriate bucket in a routing table
    /// (number of leading bits that are identical).
    fn bucket_index(&self, other: &Self) -> usize;
}

/// A `Bucket` is a collection of `Node`s that are sorted by their `NodeId`.
/// The `Bucket` is used in a `RoutingTable` to store nodes that are close to
/// each others.
pub struct Bucket<A: Address, N: NodeId> {
    // The nodes are sorted by node id.
    nodes: Vec<Node<A, N>>,
}

/// A `Node` is a representation of a node in a distributed system. It contains
/// the node's `NodeId` and a list of `Address`es that can be used to contact
/// the node.
pub struct Node<A: Address, N: NodeId> {
    id: N,
    addresses: Vec<A>,
}

/// A `RoutingTable` stores a collection of `Bucket`s that contain `Node`s. The
/// `RoutingTable` is used in a distributed system to keep track of nodes that
/// are close to each other in the network.
pub struct RoutingTable<A: Address, N: NodeId> {
    buckets: Vec<Bucket<A, N>>,
    local_id: N,
    bucket_size: usize,
}

impl<A: Address, N: NodeId> Bucket<A, N> {

    /// Get the first node in the bucket.
    pub fn first(&self) -> Option<&Node<A, N>> {
        self.nodes.first()
    }

    /// Get the last node in the bucket.
    pub fn last(&self) -> Option<&Node<A, N>> {
        self.nodes.last()
    }

    /// Get the node at the given index.
    pub fn get(&self, index: usize) -> Option<&Node<A, N>> {
        self.nodes.get(index)
    }

    /// Get a mutable reference to the node at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Node<A, N>> {
        self.nodes.get_mut(index)
    }

    /// Find the index of the node with the given id.
    fn find(&self, id: &N) -> Result<usize, usize> {
        self.nodes.binary_search_by(|node| node.id.cmp(id))
    }

    /// Insert a node into the bucket. 
    /// 
    /// If the node is already in the bucket, it will not be inserted.
    pub fn insert(&mut self, node: Node<A, N>) -> bool {
        match self.find(&node.id) {
            Ok(_) => false,
            Err(index) => {
                self.nodes.insert(index, node);
                true
            }
        }
    }

    /// Remove the node with the given id from the bucket.
    /// 
    /// Returns the removed node if it was found, otherwise None.
    pub fn remove(&mut self, id: &N) -> Option<Node<A, N>> {
        match self.find(id) {
            Ok(index) => {
                Some(self.nodes.remove(index))
            }
            Err(_) => None,
        }
    }

    /// Check if the bucket contains a node with the given id.
    /// 
    /// Returns true if the node is found, otherwise false.
    pub fn contains(&self, id: &N) -> bool {
        self.find(id).is_ok()
    }

    /// Check if the node with the given id is within the range of the bucket.
    /// 
    /// Returns true if the node is within the range, otherwise false.
    /// 
    /// TODO: Might not work as wanted, need to test.
    pub fn range_contains(&self, id: &N) -> bool {
        let first = self.first().expect("Bucket is empty");
        let last = self.last().expect("Bucket is empty");
        id.cmp_distance(&first.id) != Ordering::Greater
            && id.cmp_distance(&last.id) != Ordering::Less
    }

    /// Get the number of nodes in the bucket.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl<A: Address, N: NodeId> RoutingTable<A, N> {
    /// Create a new `RoutingTable` with the given local id.
    /// 
    /// The `local_id` is the id of the node that owns the routing table.
    /// 
    /// The `bucket_size` is the maximum number of nodes that can be stored in a bucket. By default,
    /// the bucket size is set to 20.
    pub fn new(local_id: N) -> RoutingTable<A, N> {
        RoutingTable {
            buckets: vec![],
            local_id: local_id,
            bucket_size: 20,
        }
    }

    /// Find the index of the bucket that contains the node with the given id.
    ///
    /// Returns the index of the bucket if it is found, otherwise None.
    fn find_bucket_index(&self, id: &N) -> Option<usize> {
        if self.buckets.len() == 0 {
            return None;
        }

        let mut bucket_index = 0;
        let mut bucket_length = 0;
        for (i, bucket) in self.buckets.iter().enumerate() {
            if bucket.nodes.len() > 0 {
                let first = bucket.first().expect("Bucket is empty");
                let last = bucket.last().expect("Bucket is empty");
                let lbindex = id.bucket_index(&first.id);
                let rbindex = id.bucket_index(&last.id);
                let bindex = min(lbindex, rbindex);
                if bindex >= bucket_length {
                    bucket_index = i;
                    bucket_length = bindex;
                }
            }
        }
        return Some(bucket_index);
    }

    /// Find the bucket that contains the node with the given id.
    pub fn find_bucket(&self, id: &N) -> Option<&Bucket<A, N>> {
        match self.find_bucket_index(id) {
            Some(index) => Some(&self.buckets[index]),
            None => None,
        }
    }

    /// Find the mutable reference to the bucket that contains the node with the given id.
    fn find_bucket_mut(&mut self, id: &N) -> Option<&mut Bucket<A, N>> {
        match self.find_bucket_index(id) {
            Some(index) => Some(&mut self.buckets[index]),
            None => None,
        }
    }

    /// Insert a node into the routing table.
    /// 
    /// Returns true if the node was inserted, otherwise false.
    /// 
    /// If the bucket that contains the node is full, it will be split into two new buckets
    /// if the local id is within the range of the bucket. Otherwise, the node will not be inserted.
    pub fn insert(&mut self, node: Node<A, N>) -> bool {
        let bucket_size = self.bucket_size;
        let local_id = self.local_id.clone();
        let node_id = node.id.clone();
        let bucket = self.find_bucket_mut(&node.id);
        let must_split;
        match bucket {
            Some(bucket) => {
                if bucket.nodes.len() >= bucket_size {
                    // TODO: Not sure if this is correct
                    if bucket.range_contains(&local_id) {
                        bucket.insert(node);
                        must_split = true;
                    } else {
                        return false;
                    }
                } else {
                    return bucket.insert(node);
                }
            }
            None => {
                let new_bucket = Bucket { nodes: vec![node] };
                self.buckets.push(new_bucket);
                must_split = false;
            }
        }
        if must_split {
            self.split_bucket(self.find_bucket_index(&node_id).expect("Bucket not found"));
        }
        return true;
    }

    /// Split the bucket at the given index into two new buckets.
    /// 
    /// The bucket will be split into two new buckets based on the range of the node ids.
    /// The new buckets will be inserted into the routing table, and the old bucket will be removed.
    fn split_bucket(&mut self, index: usize) {
        let bucket = self.buckets.remove(index);
        if bucket.nodes.len() < self.bucket_size {
            self.buckets.push(bucket);
            return;
        }

        let mut left = Bucket { nodes: vec![] };
        let mut right = Bucket { nodes: vec![] };
        let first_id = bucket.first().expect("Bucket is empty").id.clone();
        let last_id = bucket.last().expect("Bucket is empty").id.clone();
        let bucket_index = first_id.bucket_index(&last_id);
        for node in bucket.nodes {
            let index = first_id.bucket_index(&node.id);
            if index >= bucket_index {
                left.insert(node);
            } else {
                right.insert(node);
            }
        }
        self.buckets.push(left);
        self.buckets.push(right);
    }

    /// Remove the node with the given id from the routing table.
    /// 
    /// Returns the removed node if it was found, otherwise None.
    /// 
    /// If the bucket that contains the node is empty after removing the node, it will be removed.
    pub fn remove(&mut self, id: &N) -> Option<Node<A, N>> {
        let bucket_index = self.find_bucket_index(id);
        match bucket_index {
            Some(index) => {
                let bucket = &mut self.buckets[index];
                let node = bucket.remove(id);
                if bucket.len() == 0 {
                    self.buckets.remove(index);
                }
                node
            }
            None => None,
        }
    }
}
