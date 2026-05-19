use std::collections::{BTreeSet, HashMap, HashSet};

use crate::game::{BoardDefinition, NodeId};

#[derive(Debug, Clone)]
pub struct Graph {
    pub name: &'static str,
    pub nodes: HashMap<NodeId, crate::game::BoardNode>,
    pub adjacency: HashMap<NodeId, Vec<NodeId>>,
    pub turn_limit: u32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum BoardError {
    DuplicateNodeId(NodeId),
    UnknownNode(NodeId),
    DuplicateEdge(NodeId, NodeId),
    SelfEdge(NodeId),
    InvalidStart(String),
}

impl std::fmt::Display for BoardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardError::DuplicateNodeId(id) => write!(f, "duplicate node id: {id}"),
            BoardError::UnknownNode(id) => write!(f, "edge references unknown node id: {id}"),
            BoardError::DuplicateEdge(a, b) => write!(f, "duplicate edge: {a} -- {b}"),
            BoardError::SelfEdge(id) => write!(f, "self edge is not allowed: {id} -- {id}"),
            BoardError::InvalidStart(msg) => write!(f, "invalid start state: {msg}"),
        }
    }
}

impl std::error::Error for BoardError {}

impl Graph {
    pub fn from_board_definition(board: &BoardDefinition) -> Result<Self, BoardError> {
        let mut nodes = HashMap::new();
        let mut coord_set = HashSet::new();
        let mut warnings = Vec::new();

        for node in board.nodes {
            if nodes.insert(node.id, *node).is_some() {
                return Err(BoardError::DuplicateNodeId(node.id));
            }
            if !coord_set.insert((node.x, node.y)) {
                warnings.push(format!(
                    "warning: duplicate coordinate detected at ({}, {})",
                    node.x, node.y
                ));
            }
        }

        let mut adjacency_set: HashMap<NodeId, BTreeSet<NodeId>> = nodes
            .keys()
            .copied()
            .map(|id| (id, BTreeSet::new()))
            .collect();

        let mut normalized_edges = HashSet::new();
        for edge in board.edges {
            let (a, b) = (edge.0, edge.1);
            if a == b {
                return Err(BoardError::SelfEdge(a));
            }
            if !nodes.contains_key(a) {
                return Err(BoardError::UnknownNode(a));
            }
            if !nodes.contains_key(b) {
                return Err(BoardError::UnknownNode(b));
            }
            let normalized = if a < b { (a, b) } else { (b, a) };
            if !normalized_edges.insert(normalized) {
                return Err(BoardError::DuplicateEdge(normalized.0, normalized.1));
            }
            adjacency_set.entry(a).or_default().insert(b);
            adjacency_set.entry(b).or_default().insert(a);
        }

        validate_start(board, &nodes)?;

        let adjacency = adjacency_set
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect::<Vec<_>>()))
            .collect();

        Ok(Self {
            name: board.name,
            nodes,
            adjacency,
            turn_limit: board.turn_limit,
            warnings,
        })
    }

    pub fn neighbors(&self, node: NodeId) -> &[NodeId] {
        self.adjacency
            .get(node)
            .map(Vec::as_slice)
            .expect("unknown node in graph")
    }
}

fn validate_start(
    board: &BoardDefinition,
    nodes: &HashMap<NodeId, crate::game::BoardNode>,
) -> Result<(), BoardError> {
    if !nodes.contains_key(board.start.bear) {
        return Err(BoardError::InvalidStart(format!(
            "bear start node '{}' does not exist",
            board.start.bear
        )));
    }
    let mut seen = HashSet::new();
    for hunter in board.start.hunters {
        if !nodes.contains_key(hunter) {
            return Err(BoardError::InvalidStart(format!(
                "hunter start node '{}' does not exist",
                hunter
            )));
        }
        if !seen.insert(hunter) {
            return Err(BoardError::InvalidStart(format!(
                "duplicate hunter start node '{}'",
                hunter
            )));
        }
        if hunter == board.start.bear {
            return Err(BoardError::InvalidStart(
                "bear and hunter cannot share a start node".to_string(),
            ));
        }
    }
    Ok(())
}
