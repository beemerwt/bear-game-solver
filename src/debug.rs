use crate::{game::GameState, graph::Graph};

pub fn render_debug_board(graph: &Graph, state: Option<&GameState>) -> String {
    let mut nodes = graph.nodes.values().copied().collect::<Vec<_>>();
    nodes.sort_by_key(|n| (n.y, n.x));

    let mut out = String::from("Debug board listing (positive Y is down):\n");
    for node in nodes {
        let occupant = match state {
            Some(s) if s.bear == node.id => "bear",
            Some(s) if s.hunters.contains(&node.id) => "hunter",
            _ => "empty",
        };
        let mut neighbors = graph.neighbors(node.id).to_vec();
        neighbors.sort_unstable();
        out.push_str(&format!(
            "- {:<18} ({:>2}, {:>2}) {:<6} -> {:?}\n",
            node.id, node.x, node.y, occupant, neighbors
        ));
    }
    out
}
