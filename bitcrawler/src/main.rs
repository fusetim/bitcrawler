use std::{fmt::Display, net::{Ipv4Addr, UdpSocket}, time::{Duration, Instant, UNIX_EPOCH}};

use bitcrawler_proto::{bencode, kademlia::{NodeId, Xorable}, krpc::{node_info, query::Ping, Query, Response, ResponseType}};


const DHT_BOOTSTRAP: (&str, u16) = ("dht.transmissionbt.com", 6881);
const DHT_PORT: u16 = 6881;
const NODE_ID: BittorrentNodeId = BittorrentNodeId([0,1,2,3,4,5,6,7,8,9,99,98,97,96,95,94,93,92,91,90]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BittorrentNodeId(pub [u8; 20]);

impl Xorable for BittorrentNodeId {
    fn cmp_distance(&self, other: &Self) -> std::cmp::Ordering {
        return self.0.cmp(&other.0);
    }

    fn bucket_index(&self, other: &Self) -> usize {
        for i in 0..self.0.len() {
            if self.0[i] != other.0[i] {
                return i;
            }
        }
        return self.0.len();
    }
}

impl<'a> TryFrom<&'a [u8]> for BittorrentNodeId {
    type Error = &'static str;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() != 20 {
            return Err("Invalid length for BittorrentNodeId");
        }
        let mut node_id = [0u8; 20];
        node_id.copy_from_slice(value);
        Ok(BittorrentNodeId(node_id))
    }
}

impl Into<Vec<u8>> for BittorrentNodeId {
    fn into(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Display for BittorrentNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Convert the node ID to a hexadecimal string
        let hex_string = self.0.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
        // Write the hexadecimal string to the formatter
        write!(f, "{}", hex_string)
    }
}

impl NodeId for BittorrentNodeId {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BittorrentNodeInfoV4 {
    pub node_id: BittorrentNodeId,
    pub address: IPv4Address,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IPv4Address {
    pub ip: [u8; 4],
    pub port: u16,
}

impl node_info::NodeInfo for BittorrentNodeInfoV4 {
    type NodeId = BittorrentNodeId;
    type Address = IPv4Address;

    fn get_node_id(&self) -> &Self::NodeId {
        &self.node_id
    }

    fn to_address(&self) -> Self::Address {
        IPv4Address {
            ip: self.address.ip,
            port: self.address.port,
        }
    }

    fn new_with_address(node_id: Self::NodeId, address: Self::Address) -> Self {
        BittorrentNodeInfoV4 {
            node_id,
            address,
        }
    }
}

impl node_info::CompactNodeInfo for BittorrentNodeInfoV4 {
    type Error = &'static str;

    fn try_read_compact_node_info(data: &[u8]) -> Result<(usize, Self), Self::Error> {
        if data.len() < 26 {
            return Err("Invalid length for compact node info");
        }
        let mut node_id = [0u8; 20];
        node_id.copy_from_slice(&data[0..20]);
        let ip = [data[20], data[21], data[22], data[23]];
        let port = u16::from_be_bytes([data[24], data[25]]);
        Ok((
            26,
            BittorrentNodeInfoV4 {
                node_id: BittorrentNodeId(node_id),
                address: IPv4Address { ip, port },
            },
        ))
    }

    fn write_compact_node_info(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(26);
        data.extend_from_slice(&self.node_id.0);
        data.extend_from_slice(&self.address.ip);
        data.extend_from_slice(&self.address.port.to_be_bytes());
        data
    }
}

fn main() {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, DHT_PORT)).unwrap();
    println!("Listening on {:?}", socket.local_addr().unwrap());
    socket.set_read_timeout(Some(Duration::new(1, 0))).unwrap();

    let reference_zero = Instant::now();

    let mut sent = Instant::now();
    let mut buf = [0; 1024];
    loop {
        if let Ok((size, src)) = socket.recv_from(&mut buf) {
            let data = &buf[..size];

            if let Ok((_, response)) = bencode::decode(&data) {
                let ping_response = match Response::<BittorrentNodeInfoV4>::try_from_ping_bencoded(&response) {
                    Ok(ping_response) => ping_response,
                    Err(_) => {
                        println!("received: {:?}", response);
                        continue;
                    },
                };
    
                match ping_response.get_response_type() {
                    ResponseType::Ping(ping) => {
                        let tid = ping_response.get_transaction_id();
                        let node_id = ping.get_id();
                        let sent_time_ns = String::try_from(tid.to_owned()).unwrap().parse::<u128>().unwrap();
                        let received_time_ns = reference_zero.elapsed().as_nanos();
                        let round_trip_time = (received_time_ns - sent_time_ns) as f64 / 1000000.0;
                        println!("Ping response from {}/{:?}: RTT = {} ms", node_id, src, round_trip_time);
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }

        if sent.elapsed().as_secs() > 5 {
            sent = Instant::now();
            let current_time = sent.duration_since(reference_zero).as_nanos();
            let ping_query = Query::new_ping(current_time.to_string(), NODE_ID);
            let ping_bencoded = bencode::encode(&ping_query.to_bencoded());
            socket.send_to(&ping_bencoded, DHT_BOOTSTRAP).unwrap();
            println!("Sent ping to {:?}", DHT_BOOTSTRAP);
        }
    }
}