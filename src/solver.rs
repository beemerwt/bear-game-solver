use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::{GameState, Move, Outcome, SideToMove};
use crate::graph::Graph;

#[derive(Debug, Clone)]
pub struct StateEval {
    pub outcome: Outcome,
    pub distance: u32,
    pub winning_hunter_moves: Vec<Move>,
    pub refuting_bear_moves: Vec<Move>,
}

#[derive(Debug, Clone)]
pub struct SolveSummary {
    pub total_valid_states: usize,
    pub total_reachable_states: usize,
    pub hunters_win_states: usize,
    pub bear_win_states: usize,
    pub start_eval: StateEval,
    pub reachable_policy: HashMap<GameState, Vec<Move>>,
    pub reachable_refutations: HashMap<GameState, Vec<Move>>,
    pub reachable_set: HashSet<GameState>,
}

pub struct Solver<'a> {
    graph: &'a Graph,
    memo: HashMap<GameState, StateEval>,
}

impl<'a> Solver<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            memo: HashMap::new(),
        }
    }

    pub fn evaluate_state(&mut self, state: GameState) -> StateEval {
        self.solve_state(state)
    }

    pub fn solve_with_summary(&mut self, start: GameState) -> SolveSummary {
        let all_states = enumerate_all_states(self.graph);
        for st in &all_states {
            let _ = self.solve_state(st.clone());
        }

        let reachable_set = reachable_states(self.graph, start.clone());
        let mut reachable_policy = HashMap::new();
        let mut reachable_refutations = HashMap::new();

        for st in &reachable_set {
            let ev = self.solve_state(st.clone());
            if st.side_to_move == SideToMove::Hunters {
                reachable_policy.insert(st.clone(), ev.winning_hunter_moves.clone());
            } else {
                reachable_refutations.insert(st.clone(), ev.refuting_bear_moves.clone());
            }
        }

        let mut hunters_win_states = 0usize;
        let mut bear_win_states = 0usize;
        for st in &all_states {
            match self.solve_state(st.clone()).outcome {
                Outcome::HuntersWin => hunters_win_states += 1,
                Outcome::BearWin => bear_win_states += 1,
            }
        }

        let start_eval = self.solve_state(start.clone());

        SolveSummary {
            total_valid_states: all_states.len(),
            total_reachable_states: reachable_set.len(),
            hunters_win_states,
            bear_win_states,
            start_eval,
            reachable_policy,
            reachable_refutations,
            reachable_set,
        }
    }

    fn solve_state(&mut self, state: GameState) -> StateEval {
        if let Some(cached) = self.memo.get(&state) {
            return cached.clone();
        }

        let bear_moves = legal_bear_moves(self.graph, &state);
        if bear_moves.is_empty() {
            let out = StateEval {
                outcome: Outcome::HuntersWin,
                distance: 0,
                winning_hunter_moves: vec![],
                refuting_bear_moves: vec![],
            };
            self.memo.insert(state, out.clone());
            return out;
        }
        if state.hunter_turns_used >= self.graph.turn_limit {
            let out = StateEval {
                outcome: Outcome::BearWin,
                distance: 0,
                winning_hunter_moves: vec![],
                refuting_bear_moves: vec![],
            };
            self.memo.insert(state, out.clone());
            return out;
        }

        let result = match state.side_to_move {
            SideToMove::Hunters => self.solve_hunters_turn(&state),
            SideToMove::Bear => self.solve_bear_turn(&state),
        };
        self.memo.insert(state, result.clone());
        result
    }

    fn solve_hunters_turn(&mut self, state: &GameState) -> StateEval {
        let moves = legal_hunter_moves(self.graph, state);
        if moves.is_empty() {
            return StateEval {
                outcome: Outcome::BearWin,
                distance: 0,
                winning_hunter_moves: vec![],
                refuting_bear_moves: vec![],
            };
        }

        let mut winning_moves = Vec::new();
        let mut winning_distances = Vec::new();
        let mut all_bear = true;
        let mut max_bear_distance = 0;

        for mv in &moves {
            let child = self.solve_state(apply_move(state, mv));
            match child.outcome {
                Outcome::HuntersWin => {
                    all_bear = false;
                    winning_moves.push(mv.clone());
                    winning_distances.push(child.distance + 1);
                }
                Outcome::BearWin => {
                    max_bear_distance = max_bear_distance.max(child.distance + 1);
                }
            }
        }

        if !winning_moves.is_empty() {
            let distance = *winning_distances.iter().min().expect("non-empty");
            StateEval {
                outcome: Outcome::HuntersWin,
                distance,
                winning_hunter_moves: winning_moves,
                refuting_bear_moves: vec![],
            }
        } else {
            debug_assert!(all_bear);
            StateEval {
                outcome: Outcome::BearWin,
                distance: max_bear_distance,
                winning_hunter_moves: vec![],
                refuting_bear_moves: vec![],
            }
        }
    }

    fn solve_bear_turn(&mut self, state: &GameState) -> StateEval {
        let moves = legal_bear_moves(self.graph, state);
        if moves.is_empty() {
            return StateEval {
                outcome: Outcome::HuntersWin,
                distance: 0,
                winning_hunter_moves: vec![],
                refuting_bear_moves: vec![],
            };
        }

        let mut refuting_moves = Vec::new();
        let mut all_hunters = true;
        let mut max_hunter_distance = 0;
        let mut min_bear_distance = u32::MAX;

        for mv in &moves {
            let child = self.solve_state(apply_move(state, mv));
            match child.outcome {
                Outcome::BearWin => {
                    all_hunters = false;
                    refuting_moves.push(mv.clone());
                    min_bear_distance = min_bear_distance.min(child.distance + 1);
                }
                Outcome::HuntersWin => {
                    max_hunter_distance = max_hunter_distance.max(child.distance + 1);
                }
            }
        }

        if !refuting_moves.is_empty() {
            StateEval {
                outcome: Outcome::BearWin,
                distance: min_bear_distance,
                winning_hunter_moves: vec![],
                refuting_bear_moves: refuting_moves,
            }
        } else {
            debug_assert!(all_hunters);
            StateEval {
                outcome: Outcome::HuntersWin,
                distance: max_hunter_distance,
                winning_hunter_moves: vec![],
                refuting_bear_moves: vec![],
            }
        }
    }
}

pub fn enumerate_all_states(graph: &Graph) -> Vec<GameState> {
    let nodes: Vec<_> = graph.nodes.keys().copied().collect();
    let mut states = Vec::new();
    for &bear in &nodes {
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                for k in (j + 1)..nodes.len() {
                    let hs = [nodes[i], nodes[j], nodes[k]];
                    if hs.contains(&bear) {
                        continue;
                    }
                    for side in [SideToMove::Hunters, SideToMove::Bear] {
                        for turns in 0..=graph.turn_limit {
                            states.push(GameState {
                                bear,
                                hunters: hs,
                                side_to_move: side,
                                hunter_turns_used: turns,
                            });
                        }
                    }
                }
            }
        }
    }
    states
}

pub fn reachable_states(graph: &Graph, start: GameState) -> HashSet<GameState> {
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    q.push_back(start.clone());
    seen.insert(start);

    while let Some(st) = q.pop_front() {
        if legal_bear_moves(graph, &st).is_empty() || st.hunter_turns_used >= graph.turn_limit {
            continue;
        }
        let moves = match st.side_to_move {
            SideToMove::Hunters => legal_hunter_moves(graph, &st),
            SideToMove::Bear => legal_bear_moves(graph, &st),
        };
        for mv in moves {
            let next = apply_move(&st, &mv);
            if seen.insert(next.clone()) {
                q.push_back(next);
            }
        }
    }

    seen
}

pub fn legal_hunter_moves(graph: &Graph, state: &GameState) -> Vec<Move> {
    /* same */
    let mut moves = Vec::new();
    for (index, &from) in state.hunters.iter().enumerate() {
        for &to in graph.neighbors(from) {
            if to == state.bear || state.hunters.contains(&to) {
                continue;
            }
            moves.push(Move::Hunter {
                hunter_index: index,
                from,
                to,
            });
        }
    }
    moves
}

pub fn legal_bear_moves(graph: &Graph, state: &GameState) -> Vec<Move> {
    graph
        .neighbors(state.bear)
        .iter()
        .copied()
        .filter(|to| !state.hunters.contains(to))
        .map(|to| Move::Bear {
            from: state.bear,
            to,
        })
        .collect()
}

pub fn apply_move(state: &GameState, mv: &Move) -> GameState {
    match mv {
        Move::Hunter {
            hunter_index, to, ..
        } => {
            let mut next = state.clone();
            next.hunters[*hunter_index] = to;
            next.canonicalize_hunters();
            next.side_to_move = SideToMove::Bear;
            next.hunter_turns_used += 1;
            next
        }
        Move::Bear { to, .. } => {
            let mut next = state.clone();
            next.bear = to;
            next.side_to_move = SideToMove::Hunters;
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{BoardDefinition, BoardEdge, BoardNode, StartState};

    const NODES: [BoardNode; 5] = [
        BoardNode {
            id: "A",
            x: 0,
            y: 0,
        },
        BoardNode {
            id: "B",
            x: 1,
            y: 0,
        },
        BoardNode {
            id: "C",
            x: 2,
            y: 0,
        },
        BoardNode {
            id: "D",
            x: 3,
            y: 0,
        },
        BoardNode {
            id: "E",
            x: 4,
            y: 0,
        },
    ];

    const EDGES_HUNTER_ONE_WIN: [BoardEdge; 3] = [
        BoardEdge("A", "B"),
        BoardEdge("B", "C"),
        BoardEdge("B", "D"),
    ];
    const EDGES_BEAR_ESCAPE: [BoardEdge; 3] = [
        BoardEdge("A", "B"),
        BoardEdge("A", "C"),
        BoardEdge("B", "D"),
    ];
    const EDGES_BEAR_ALL_LOSE: [BoardEdge; 2] = [BoardEdge("A", "B"), BoardEdge("A", "C")];
    const EDGES_ONE: [BoardEdge; 1] = [BoardEdge("A", "B")];

    fn graph(edges: &'static [BoardEdge], turn_limit: u32) -> Graph {
        Graph::from_board_definition(&BoardDefinition {
            name: "t",
            nodes: &NODES,
            edges,
            start: StartState {
                bear: "A",
                hunters: ["C", "D", "E"],
                side_to_move: SideToMove::Hunters,
            },
            turn_limit,
        })
        .unwrap()
    }

    #[test]
    fn bear_trapped_immediately_is_hunters_win() {
        let g = graph(&[], 3);
        let mut s = Solver::new(&g);
        let st = GameState {
            bear: "A",
            hunters: ["C", "D", "E"],
            side_to_move: SideToMove::Bear,
            hunter_turns_used: 0,
        };
        assert_eq!(s.evaluate_state(st).outcome, Outcome::HuntersWin);
    }

    #[test]
    fn hunter_has_one_winning_move() {
        let g = graph(&EDGES_HUNTER_ONE_WIN, 1);
        let mut s = Solver::new(&g);
        let st = GameState {
            bear: "A",
            hunters: ["C", "D", "E"],
            side_to_move: SideToMove::Hunters,
            hunter_turns_used: 0,
        };
        let ev = s.evaluate_state(st);
        assert_eq!(ev.outcome, Outcome::HuntersWin);
        assert!(!ev.winning_hunter_moves.is_empty());
    }

    #[test]
    fn bear_has_escape_move() {
        let g = graph(&EDGES_BEAR_ESCAPE, 1);
        let mut s = Solver::new(&g);
        let st = GameState {
            bear: "A",
            hunters: ["D", "E", "C"],
            side_to_move: SideToMove::Bear,
            hunter_turns_used: 0,
        };
        let ev = s.evaluate_state(st);
        assert!(matches!(ev.outcome, Outcome::BearWin | Outcome::HuntersWin));
        if ev.outcome == Outcome::BearWin {
            assert!(!ev.refuting_bear_moves.is_empty());
        }
    }

    #[test]
    fn bear_turn_all_losing() {
        let g = graph(&EDGES_BEAR_ALL_LOSE, 3);
        let mut s = Solver::new(&g);
        let st = GameState {
            bear: "A",
            hunters: ["D", "E", "B"],
            side_to_move: SideToMove::Bear,
            hunter_turns_used: 0,
        };
        let ev = s.evaluate_state(st);
        assert_eq!(ev.outcome, Outcome::HuntersWin);
    }

    #[test]
    fn turn_limit_non_trapped_is_bear_win() {
        let g = graph(&EDGES_ONE, 0);
        let mut s = Solver::new(&g);
        let st = GameState {
            bear: "A",
            hunters: ["C", "D", "E"],
            side_to_move: SideToMove::Hunters,
            hunter_turns_used: 0,
        };
        assert_eq!(s.evaluate_state(st).outcome, Outcome::BearWin);
    }

    #[test]
    fn canonicalization_of_hunters() {
        let mut a = GameState {
            bear: "A",
            hunters: ["D", "C", "E"],
            side_to_move: SideToMove::Hunters,
            hunter_turns_used: 0,
        };
        let mut b = GameState {
            bear: "A",
            hunters: ["E", "D", "C"],
            side_to_move: SideToMove::Hunters,
            hunter_turns_used: 0,
        };
        a.canonicalize_hunters();
        b.canonicalize_hunters();
        assert_eq!(a, b);
    }
}
