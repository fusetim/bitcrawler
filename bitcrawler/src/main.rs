use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, Write},
    net::{Ipv4Addr, UdpSocket},
    thread::sleep,
    time::{Duration, Instant, UNIX_EPOCH},
};

use bitcrawler_proto::{
    bencode,
    kademlia::{NodeId, Xorable},
    krpc::{
        Query, Response, ResponseType, node_info,
        peer_info::CompactPeerInfo,
        query::{Ping, QUERY_TYPE_FIND_NODE, QUERY_TYPE_GET_PEERS, QUERY_TYPE_PING},
    },
};

const DHT_BOOTSTRAP: (&str, u16) = ("77.234.80.66", 29822);
const DHT_PORT: u16 = 6881;
const NODE_ID: BittorrentNodeId = BittorrentNodeId([
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 99, 98, 97, 96, 95, 94, 93, 92, 91, 90,
]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        let hex_string = self
            .0
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        // Write the hexadecimal string to the formatter
        write!(f, "{}", hex_string)
    }
}

impl NodeId for BittorrentNodeId {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BittorrentNodeInfoV4 {
    pub node_id: BittorrentNodeId,
    pub address: IPv4Address,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IPv4Address {
    pub ip: [u8; 4],
    pub port: u16,
}

impl TryFrom<&str> for IPv4Address {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid IPv4 address format");
        }
        let ip_parts: Vec<u8> = parts[0]
            .split('.')
            .map(|s| s.parse::<u8>().unwrap_or(0))
            .collect();
        if ip_parts.len() != 4 {
            return Err("Invalid IPv4 address format");
        }
        let port = parts[1].parse::<u16>().unwrap_or(0);
        Ok(IPv4Address {
            ip: [ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]],
            port,
        })
    }
}

impl CompactPeerInfo for IPv4Address {
    type Error = &'static str;

    fn try_read_compact_peer_info(data: &[u8]) -> Result<(usize, Self), Self::Error> {
        if data.len() < 6 {
            return Err("Invalid length for compact peer info");
        }
        let ip = [data[0], data[1], data[2], data[3]];
        let port = u16::from_be_bytes([data[4], data[5]]);
        Ok((6, IPv4Address { ip, port }))
    }

    fn write_compact_peer_info(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(6);
        data.extend_from_slice(&self.ip);
        data.extend_from_slice(&self.port.to_be_bytes());
        data
    }
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
        BittorrentNodeInfoV4 { node_id, address }
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
    let lookup_hash = BittorrentNodeId([
        0x00, 0xab, 0xb5, 0xd1, 0x2f, 0xb0, 0x3c, 0x7e, 0xe2, 0x88, 0x76, 0x78, 0x9c, 0x43, 0xeb, 0xe2, 0x6d, 0x36, 0xe0, 0xa1
    ]);


    let mut contacts: Vec<IPv4Address> = Vec::new();
    let mut seen = HashSet::new();
    let mut sent = Instant::now();
    let mut buf = [0; 1024];

    // Load previously discovered nodes from the file
    if let Ok(node_list_file) = File::open("/tmp/node_list.txt") {
        let reader = BufReader::new(&node_list_file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(contact) = IPv4Address::try_from(line.as_str()) {
                    contacts.push(contact);
                }
            }
        }
        println!("Loaded {} nodes from file", contacts.len());
    }
    // Open and truncate the file for writing
    let mut node_list_file = File::create("/tmp/node_list.txt").unwrap();


    loop {
        if let Ok((size, src)) = socket.recv_from(&mut buf) {
            let data = &buf[..size];

            if let Ok((_, response)) = bencode::decode(&data) {
                let response__ = match Response::<BittorrentNodeInfoV4, IPv4Address>::try_guess_type_from_bencoded(&response) {
                    Ok((query_type, _)) => match query_type {
                        QUERY_TYPE_PING => {
                            Response::try_from_ping_bencoded(&response).unwrap()
                        }
                        QUERY_TYPE_GET_PEERS | QUERY_TYPE_FIND_NODE => {
                            Response::try_from_getpeers_bencoded(&response).unwrap()
                        }
                        _ => continue,
                    },
                    Err(_) => {
                        println!("Failed to decode response");
                        continue;
                    }
                };

                match response__.get_response_type() {
                    ResponseType::Ping(ping) => {
                        let tid = response__.get_transaction_id();
                        let node_id: &BittorrentNodeId = ping.get_id();
                        let sent_time_ns = String::try_from(tid.to_owned())
                            .unwrap()
                            .parse::<u128>()
                            .unwrap();
                        let received_time_ns = reference_zero.elapsed().as_nanos();
                        let round_trip_time = (received_time_ns - sent_time_ns) as f64 / 1000000.0;
                        /*println!(
                            "Ping response from {}/{:?}: RTT = {} ms",
                            node_id, src, round_trip_time
                        );*/

                        seen.insert(node_id.clone());

                        // Node is available, asked for other nodes for lookup_hash
                        let lookup_query = Query::new_get_peers(
                            received_time_ns.to_string(),
                            NODE_ID,
                            lookup_hash.clone(),
                        );
                        let lookup_bencoded = bencode::encode(&lookup_query.to_bencoded());
                        socket.send_to(&lookup_bencoded, src).unwrap();
                        //println!("Sent lookup query to {:?}", src);
                    }
                    ResponseType::GetPeers(getpeers) => {
                        let tid = response__.get_transaction_id();
                        let node_id = getpeers.get_id();
                        let peers: &[IPv4Address] = getpeers.get_peers();
                        let nodes: &[BittorrentNodeInfoV4] = getpeers.get_nodes();
                        println!(
                            "GetPeers response from {}/{:?}: {} peers, {} nodes",
                            node_id,
                            src,
                            peers.len(),
                            nodes.len()
                        );
                        for node in nodes {
                            if (node.node_id != NODE_ID) && (&node.node_id != node_id) {
                                if seen.insert(node.node_id.clone()) {
                                    contacts.push(IPv4Address {
                                        ip: node.address.ip,
                                        port: node.address.port,
                                    });
                                    node_list_file
                                        .write_all(
                                            format!(
                                                "{}.{}.{}.{}:{}\n",
                                                node.address.ip[0],
                                                node.address.ip[1],
                                                node.address.ip[2],
                                                node.address.ip[3],
                                                node.address.port
                                            )
                                            .as_bytes(),
                                        )
                                        .unwrap();
                                }
                            }
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }

        if sent.elapsed().as_secs() > 2 {
            sent = Instant::now();
            let current_time = sent.duration_since(reference_zero).as_nanos();
            let ping_query = Query::new_ping(current_time.to_string(), NODE_ID);
            let ping_bencoded = bencode::encode(&ping_query.to_bencoded());
            if contacts.is_empty() {
                socket.send_to(&ping_bencoded, DHT_BOOTSTRAP).unwrap();
                println!("Sent ping to {:?}", DHT_BOOTSTRAP);
            } else {
                let mut i = 0;
                while let Some(contact) = contacts.pop() {
                    let addr = format!(
                        "{}.{}.{}.{}",
                        contact.ip[0], contact.ip[1], contact.ip[2], contact.ip[3]
                    );
                    let port = contact.port;
                    socket.send_to(&ping_bencoded, (addr.as_str(), port)).unwrap();
                    i+=1;
                    if i >= 40 {
                        break;
                    }
                }
                println!("Sent ping to {} nodes", i);
            }
            println!("Discovered {} nodes (waiting contact: {})", seen.len(), contacts.len());
        }
        sleep(Duration::from_millis(100));
    }
}
