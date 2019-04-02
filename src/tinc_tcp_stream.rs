use std::net::TcpStream;
use std::io::Write;
use std::net::SocketAddr;
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Read};
use std::time::Duration;

#[repr(i8)]
pub enum  Request {
    All                      = -1,
    Id                       = 0,
    MetaKey                  = 1,
    Challenge                = 2,
    ChalReply                = 3,
    Ack                      = 4,
    Status                   = 5,
    Error                    = 6,
    Termreq                  = 7,
    Ping                     = 8,
    Pong                     = 9,
    AddSubnet                = 10,
    DelSubnet                = 11,
    AddEdge                  = 12,
    DelEdge                  = 13,
    KeyChanged               = 14,
    ReqKey                   = 15,
    AnsKey                   = 16,
    Packet                   = 17,
    Control                  = 18,
    ReqPubkey                = 19,
    AnsPubkey                = 20,
    SptpsPacket              = 21,
    UdpInfo                  = 22,
    MtuInfo                  = 23,
    Last                     = 24,
}

#[repr(i8)]
pub enum RequestType {
    ReqInvalid              = -1,
    ReqStop                 = 0,
    ReqReload               = 1,
    ReqRestart              = 2,
    ReqDumpNodes            = 3,
    ReqDumpEdges            = 4,
    ReqDumpSubnets          = 5,
    ReqDumpConnections      = 6,
    ReqDumpGraph            = 7,
    ReqPurge                = 8,
    ReqSetDebug             = 9,
    ReqRetry                = 10,
    ReqConnect              = 11,
    ReqDisconnect           = 12,
    ReqDumpTraffic          = 13,
    ReqPcap                 = 14,
    ReqLog                  = 15,
}

pub struct TincStream {
    stream: TcpStream,
}
impl TincStream {
    pub fn new(pid_path: &str) -> Result<Self> {
        let control_cookie = Self::parse_control_cookie(pid_path)?;
        let buf = format!("{} ^{} {}\n", 0, control_cookie, 17);
        let addr = SocketAddr::from(([127, 0, 0, 1], 50069));
        let stream = TcpStream::connect(&addr)?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(400)));
        let mut tinc_stream = TincStream{stream};
        tinc_stream.send_line(buf.as_bytes())?;
        let _res = tinc_stream.recv()?;
        return Ok(tinc_stream);
    }

    fn send_line(&mut self, buf: &[u8]) -> Result<()> {
        self.stream.write(buf)?;
        return Ok(());
    }

    fn parse_control_cookie(path: &str) -> Result<String> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let iter: Vec<&str> = contents.split_whitespace().collect();
        let control_cookie = iter[1];
        return Ok(control_cookie.to_string());
    }

    pub fn stop(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqStop as i8);
        self.send_line(cmd.as_bytes())?;
        return Ok(());
    }

    pub fn reload(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqReload as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqReload as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Reload failed."));
    }

    pub fn restart(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqRestart as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqReload as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Restart failed."));
    }

    pub fn dump_nodes(&mut self) -> Result<Vec<SourceNode>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpNodes as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpNodes as i8) {
            if let Ok(source_node) = SourceNode::from_nodes(&res) {
                return Ok(source_node);
            }
            else {
                error!("dump_nodes parse source tinc node info");
            }
        }
        else {
            error!("dump_nodes check_res");
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump nodes failed."));
    }

    pub fn dump_edges(&mut self) -> Result<Vec<SourceEdge>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpEdges as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        trace!("{}", res);
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpEdges as i8) {
            if let Ok(source_edge) = SourceEdge::from_edges(&res) {
                return Ok(source_edge);
            }
            else {
                error!("dump_edges parse source tinc edges info");
            }
        }
        else {
            error!("dump_edges check_res");
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump edges failed."));
    }

    pub fn dump_subnets(&mut self) -> Result<Vec<SourceSubnet>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpSubnets as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpSubnets as i8) {
            if let Ok(source_subnet) = SourceSubnet::from_subnets(&res) {
                return Ok(source_subnet);
            }
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump subnets failed."));
    }

    pub fn dump_connections(&mut self) -> Result<Vec<SourceConnection>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpConnections as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpConnections as i8) {
            if let Ok(source_connection) = SourceConnection::from_connections(&res) {
                return Ok(source_connection);
            }
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump connections failed."));
    }

    pub fn dump_graph(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpGraph as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpGraph as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump graph failed."));
    }

    pub fn purge(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqPurge as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqPurge as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Purge failed."));
    }

    pub fn set_debug(&mut self, debug_level: i8) -> Result<()> {
        let cmd = format!("{} {} {}\n", Request::Control as i8, RequestType::ReqSetDebug as i8, debug_level);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqSetDebug as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Set debug failed."));
    }

    pub fn retry(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqRetry as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqRetry as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Retry failed."));
    }

    pub fn connect(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqConnect as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqConnect as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Connect failed."));
    }

    pub fn disconnect(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDisconnect as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDisconnect as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Disconnect failed."));
    }

    pub fn dump_traffic(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpTraffic as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpTraffic as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump traffic failed."));
    }

    pub fn pcap(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqPcap as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqPcap as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Pcap failed."));
    }

    pub fn log(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqLog as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqLog as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Log failed."));
    }

    fn recv(&mut self) -> Result<String> {
        let mut output = String::new();
        loop {
            let res = &mut [0; 1024];
            let _len = match self.stream.read(res) {
                Ok(x) => x,
                Err(_) => 0,
            };
            if _len == 0 {
                break
            }
            else {
                let this = String::from_utf8_lossy(res).to_string().replace("\u{0}", "");
                if this.len() == 0 {
                    break
                }
                else {
                    output += &this;
                }
            }
        }
        Ok(output)
    }

    fn check_res(res: &str, req: i8, req_type: i8) -> bool {
        let iter: Vec<&str> = res.split_whitespace().collect();
        let control: i8 = match iter[0].parse() {
            Ok(x) => x,
            _ => -1,
        };

        let control_type: i8 = match iter[1].parse() {
            Ok(x) => x,
            _ => -1,
        };
        if control == req && control_type == req_type {
            return true;
        }
        false
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceNode {
    pub node:                   String,
    pub id:                     String,
    pub host:                   String,
    pub _a:                     String,
    pub port:                   String,
    pub cipher:                 String,
    pub digest:                 String,
    pub maclength:              String,
    pub compression:            String,
    pub options:                String,
    pub status_int:             String,
    pub nexthop:                String,
    pub via:                    String,
    pub distance:               String,
    pub pmtu:                   String,
    pub minmtu:                 String,
    pub maxmtu:                 String,
    pub last_state_change:      String,
}
impl SourceNode {
    pub fn from(source_str: &str) -> Result<Self> {
        let source_str = source_str.to_string();
        let node_str:Vec<&str> = source_str.split(" ").collect();
        if node_str.len() == 20 {
            let node: String = node_str[2].to_string();
            let id: String = node_str[3].to_string();
            let host: String = node_str[4].to_string();
            let _a: String = node_str[5].to_string();
            let port: String = node_str[6].to_string();
            let cipher: String = node_str[7].to_string();
            let digest: String = node_str[8].to_string();
            let maclength: String = node_str[9].to_string();
            let compression: String = node_str[10].to_string();
            let options: String = node_str[11].to_string();
            let status_int: String = node_str[12].to_string();
            let nexthop: String = node_str[13].to_string();
            let via: String = node_str[14].to_string();
            let distance: String = node_str[15].to_string();
            let pmtu: String = node_str[16].to_string();
            let minmtu: String = node_str[17].to_string();
            let maxmtu: String = node_str[18].to_string();
            let last_state_change: String = node_str[19].to_string();
            let source_node = SourceNode {
                node,
                id,
                host,
                _a,
                port,
                cipher,
                digest,
                maclength,
                compression,
                options,
                status_int,
                nexthop,
                via,
                distance,
                pmtu,
                minmtu,
                maxmtu,
                last_state_change,
            };
            return Ok(source_node);
        }
        return Err(Error::new(ErrorKind::InvalidData, "Parse SourceNode failed."));
    }

    pub fn from_nodes(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let nodes_str:Vec<&str> = source_info.split("\n").collect();
        let mut nodes: Vec<SourceNode> = vec![];
        for node_str in nodes_str.clone() {
            if node_str.len() > 0 {
                if let Ok(node) = SourceNode::from(node_str) {
                    nodes.push(node)
                }
            }
        }
        return Ok(nodes);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceEdge {
    pub from:           String,
    pub to:             String,
    pub host:           String,
    pub _a:             String,
    pub port:           String,
    pub local_host:     String,
    pub _b:             String,
    pub local_port:     String,
    pub options:        String,
    pub weight:         String,
//    avg_rtt:        String,
}
impl SourceEdge{
    fn from(source_str: &str) -> Result<Self> {
        let edge_str:Vec<&str> = source_str.split(" ").collect();
        if edge_str.len() >= 12 {
            let from = edge_str[2].to_string();
            let to = edge_str[3].to_string();
            let host = edge_str[4].to_string();
            let _a = edge_str[5].to_string();
            let port = edge_str[6].to_string();
            let local_host = edge_str[7].to_string();
            let _b = edge_str[8].to_string();
            let local_port = edge_str[9].to_string();
            let options = edge_str[10].to_string();
            let weight = edge_str[11].to_string();
            return Ok(SourceEdge {
                from,
                to,
                host,
                _a,
                port,
                local_host,
                _b,
                local_port,
                options,
                weight,
            });
        }
        return Err(Error::new(ErrorKind::InvalidData, ""));
    }

    fn from_edges(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let edges_str:Vec<&str> = source_info.split("\n").collect();
        let mut edges: Vec<SourceEdge> = vec![];
        for edge_str in edges_str.clone() {
            if edge_str.len() > 0 {
                if let Ok(edge) = SourceEdge::from(edge_str) {
                    edges.push(edge)
                }
            }
        }
        return Ok(edges);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceSubnet {
    pub name:       String,
    pub addr:       String,
}
impl SourceSubnet{
    fn from(source_str: &str) -> Result<Self> {
        let subnet_str:Vec<&str> = source_str.split(" ").collect();
        if subnet_str.len() >= 4 {
            let addr = subnet_str[2].to_string();
            let name = subnet_str[3].to_string();
            return Ok(SourceSubnet {
                name,
                addr,
            });
        }
        return Err(Error::new(ErrorKind::InvalidData, ""));
    }

    fn from_subnets(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let subnets_str:Vec<&str> = source_info.split("\n").collect();
        let mut subnets: Vec<Self> = vec![];
        for subnet_str in subnets_str.clone() {
            if subnet_str.len() > 0 {
                if let Ok(subnet) = Self::from(subnet_str) {
                    subnets.push(subnet)
                }
            }
        }
        return Ok(subnets);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceConnection {
    pub node:           String,
    pub host:           String,
    pub _a:             String,
    pub port:           String,
    pub options:        String,
    pub socket:         String,
    pub status_int:     String,
}
impl SourceConnection{
    fn from(source_str: &str) -> Result<Self> {
        let connection_str:Vec<&str> = source_str.split(" ").collect();
        if connection_str.len() >= 9 {
            let node = connection_str[2].to_string();
            let host = connection_str[3].to_string();
            let _a = connection_str[4].to_string();
            let port = connection_str[5].to_string();
            let options = connection_str[6].to_string();
            let socket = connection_str[7].to_string();
            let status_int = connection_str[8].to_string();
            return Ok(SourceConnection {
                node,
                host,
                _a,
                port,
                options,
                socket,
                status_int,
            });
        }
        return Err(Error::new(ErrorKind::InvalidData, ""));
    }

    fn from_connections(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let connections_str:Vec<&str> = source_info.split("\n").collect();
        let mut connections: Vec<Self> = vec![];
        for connection_str in connections_str.clone() {
            if connection_str.len() > 0 {
                if let Ok(connection) = Self::from(connection_str) {
                    connections.push(connection)
                }
            }
        }
        return Ok(connections);
    }
}