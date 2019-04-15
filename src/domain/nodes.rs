use std::collections::HashMap;
use crate::tinc_tcp_stream::{SourceNode, SourceSubnet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub index:      u32,
    pub edges:      u32,
    pub reachable:  i32,
    pub name:       String,
    pub version:    u8,
    pub group:      u8,
    pub nets:       Vec<String>,
    pub id:         u32,
}
impl Node {
    // 解析源节点信息, 生成输出节点
    fn from(source_node: &SourceNode) -> Option<Self> {
        let status_int:i32 = hex_str_to_dec(&source_node.status_int) as i32;
        let reachable = status_int >> 4 & 1;

        // pure 分支忽略reachable为0 的节点.
        if reachable != 0 {
            return Some(Node {
                index: 0,
                edges: 0,
                reachable,
                version: 0,
                name: source_node.node.clone(),
                group: 0,
                nets: vec![],
                id: 0,
            });
        }
        None
    }

    // 分解VEC源节点信息  调用Node::from
    pub fn load_nodes(
        source_nodes: Vec<SourceNode>,
        source_subnets: Vec<SourceSubnet>
    ) -> Vec<Node> {
        let mut subnets = HashMap::new();
        for subnet in source_subnets {
            if subnets.contains_key(&subnet.name) {
                let addrs: &Vec<String> = &subnets[&subnet.name];
                let mut addrs = addrs.clone();
                addrs.push(subnet.addr);
                subnets.insert(subnet.name, addrs);
            }
            else {
                subnets.insert(subnet.name, vec![subnet.addr]);
            }
        }

        let mut nodes: Vec<Self> = vec![];

        let mut index = 0;
        for source_node in source_nodes {
            if let Some(mut node) = Self::from(&source_node) {
                index += 1;
                node.id = index;
                node.index = index;
                if subnets.contains_key(&node.name) {
                    node.nets = subnets[&node.name].clone();
                }
                nodes.push(node);
            }
        }
        return nodes;
    }
}

fn hex_str_to_dec(hex: &str) -> u32 {
    let hex_str: Vec<&str> = hex.split("").collect();
    let mut hex: Vec<u32> = vec![];
    for hex_char in hex_str {
        if hex_char.len() == 1 {
            if let Ok(x) = hex_char.parse() {
                hex.push(x);
            } else {
                let x: u32 = match hex_char {
                    _ if hex_char == "a" => 10,
                    _ if hex_char == "b" => 11,
                    _ if hex_char == "c" => 12,
                    _ if hex_char == "d" => 13,
                    _ if hex_char == "e" => 14,
                    _ if hex_char == "f" => 15,
                    _ => 0,
                };
                hex.push(x);
            }
        }
    }
    let mut out:u32 = 0;
    for i in 0..hex.len() {
        out += hex[i] * 16_u32.pow((hex.len() - i - 1) as u32);
    }
    out
}