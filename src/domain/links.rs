use std::collections::HashMap;

use crate::tinc_tcp_stream::SourceEdge;
use crate::domain::nodes::Node;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link {
    pub sname:              String,
    pub tname:              String,
    pub frac:               f64,
    pub target:             u32,
    pub weight:             u32,
    pub source:             u32,
    pub reachable:          u8,
    pub _hash:              String,
}
impl Link {
    fn from(source_edge: SourceEdge, nodes: &HashMap<String, u32>) -> Option<Self> {
        let weight = match source_edge.weight.parse() {
            Ok(x) => x,
            Err(_) => 1000,
        };
        let mut target = 0;
        if nodes.contains_key(&source_edge.to) {
            target = nodes[&source_edge.to];
        }

        let mut source = 0;
        if nodes.contains_key(&source_edge.from) {
            source = nodes[&source_edge.from];
        }

        let mut reachable = 0;
        if source != 0 && target != 0 {
            if source > target {
                return None;
            }
            reachable = 1;
        }

        let _hash = format!("{}-{}", source, target);
        Some(Link {
            sname:  source_edge.from,
            tname:  source_edge.to,
            frac:   0.0,
            target,
            weight,
            source,
            reachable,
            _hash,
        })
    }

    pub fn load_links(source_edges: Vec<SourceEdge>, nodes_info: &mut Vec<Node>) -> Vec<Link> {
        let mut links: Vec<Link> = vec![];
        let mut nodes = HashMap::new();
        for node in nodes_info.clone() {
            nodes.insert(node.name, node.index);
        }
        for edge in source_edges {
            if let Some(link) = Self::from(edge, &nodes) {
                links.push(link);
            }
        }

        let mut links_hash = HashMap::new();
        for link in &links {
            if links_hash.contains_key(&link.sname) {
                let mut num = links_hash[&link.sname];
                num += 1;
                links_hash.insert(link.sname.clone(), num);
            }
            else {
                links_hash.insert(link.sname.clone(), 1);
            }
        }
        for mut node in nodes_info {
            if links_hash.contains_key(&node.name) {
                node.edges = links_hash[&node.name];
            }
        }
        return links;
    }
}