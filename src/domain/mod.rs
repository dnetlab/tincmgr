pub mod nodes;
pub mod links;

use self::nodes::Node;
use self::links::Link;

use crate::tinc_tcp_stream::{SourceEdge, SourceSubnet, SourceNode};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    nodes: Vec<Node>,
    links: Vec<Link>
}
impl Data {
    pub fn new (
        source_nodes:       Vec<SourceNode>,
        source_subnets:     Vec<SourceSubnet>,
        source_edge:        Vec<SourceEdge>,
    ) -> Self {
        let mut nodes = Node::load_nodes(source_nodes, source_subnets);
        let links = Link::load_links(source_edge, &mut nodes);
        let mut data = Data {
            nodes,
            links,
        };
        data.frac();
        data
    }

    pub fn frac(&mut self) {
        let mut max_werght: u32 = 1;
        for link in &self.links {
            if link.weight > max_werght {
                max_werght = link.weight.clone();
            }
        }
        for i in 0..self.links.len() {
            self.links[i].frac = 1.0 - (((self.links[i].weight.clone() as f64 ) * 100.0) / (max_werght as f64)) / 100.0;
        }
    }
}