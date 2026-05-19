use std::fmt;

pub type NodeId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardNode {
    pub id: NodeId,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardEdge(pub NodeId, pub NodeId);

#[derive(Debug, Clone)]
pub struct BoardDefinition {
    pub name: &'static str,
    pub nodes: &'static [BoardNode],
    pub edges: &'static [BoardEdge],
    pub start: StartState,
    pub turn_limit: u32,
}

#[derive(Debug, Clone)]
pub struct StartState {
    pub bear: NodeId,
    pub hunters: [NodeId; 3],
    pub side_to_move: SideToMove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideToMove {
    Hunters,
    Bear,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameState {
    pub bear: NodeId,
    pub hunters: [NodeId; 3],
    pub side_to_move: SideToMove,
    pub hunter_turns_used: u32,
}

impl GameState {
    pub fn from_start(start: &StartState) -> Self {
        let mut hunters = start.hunters;
        hunters.sort_unstable();
        Self {
            bear: start.bear,
            hunters,
            side_to_move: start.side_to_move,
            hunter_turns_used: 0,
        }
    }

    pub fn canonicalize_hunters(&mut self) {
        self.hunters.sort_unstable();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Move {
    Hunter {
        hunter_index: usize,
        from: NodeId,
        to: NodeId,
    },
    Bear {
        from: NodeId,
        to: NodeId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    HuntersWin,
    BearWin,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Hunter {
                hunter_index,
                from,
                to,
            } => write!(f, "Hunter#{hunter_index}: {from} -> {to}"),
            Move::Bear { from, to } => write!(f, "Bear: {from} -> {to}"),
        }
    }
}
