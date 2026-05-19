use std::collections::HashMap;

use crate::game::{GameState, Move, Outcome, SideToMove, SolveResult};
use crate::graph::Graph;

pub struct Solver<'a> {
    graph: &'a Graph,
    memo: HashMap<GameState, SolveResult>,
}

impl<'a> Solver<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            memo: HashMap::new(),
        }
    }

    pub fn solve(&mut self, start: GameState) -> SolveResult {
        self.solve_state(start)
    }

    fn solve_state(&mut self, state: GameState) -> SolveResult {
        if let Some(cached) = self.memo.get(&state) {
            return cached.clone();
        }

        if state.hunter_turns_used >= self.graph.turn_limit {
            return SolveResult {
                outcome: Outcome::BearWin,
                distance: 0,
                best_move: None,
                principal_variation: vec![],
            };
        }

        let bear_moves = legal_bear_moves(self.graph, &state);
        if bear_moves.is_empty() {
            return SolveResult {
                outcome: Outcome::HuntersWin,
                distance: 0,
                best_move: None,
                principal_variation: vec![],
            };
        }

        let state_key = state.clone();
        let result = match state.side_to_move {
            SideToMove::Hunters => self.solve_hunters_turn(state),
            SideToMove::Bear => self.solve_bear_turn(state),
        };
        self.memo.insert(state_key, result.clone());
        result
    }

    fn solve_hunters_turn(&mut self, state: GameState) -> SolveResult {
        let moves = legal_hunter_moves(self.graph, &state);
        if moves.is_empty() {
            return SolveResult {
                outcome: Outcome::BearWin,
                distance: 0,
                best_move: None,
                principal_variation: vec![],
            };
        }

        let mut best_hunter: Option<(Move, SolveResult)> = None;
        let mut best_bear: Option<(Move, SolveResult)> = None;

        for mv in moves {
            let next = apply_move(&state, &mv);
            let child = self.solve_state(next);
            match child.outcome {
                Outcome::HuntersWin => {
                    let replace = best_hunter
                        .as_ref()
                        .map(|(_, r)| child.distance < r.distance)
                        .unwrap_or(true);
                    if replace {
                        best_hunter = Some((mv, child));
                    }
                }
                Outcome::BearWin => {
                    let replace = best_bear
                        .as_ref()
                        .map(|(_, r)| child.distance > r.distance)
                        .unwrap_or(true);
                    if replace {
                        best_bear = Some((mv, child));
                    }
                }
            }
        }

        assemble_result(best_hunter.or(best_bear).expect("at least one move"))
    }

    fn solve_bear_turn(&mut self, state: GameState) -> SolveResult {
        let moves = legal_bear_moves(self.graph, &state);
        if moves.is_empty() {
            return SolveResult {
                outcome: Outcome::HuntersWin,
                distance: 0,
                best_move: None,
                principal_variation: vec![],
            };
        }

        let mut best_bear: Option<(Move, SolveResult)> = None;
        let mut best_hunter: Option<(Move, SolveResult)> = None;

        for mv in moves {
            let next = apply_move(&state, &mv);
            let child = self.solve_state(next);
            match child.outcome {
                Outcome::BearWin => {
                    let replace = best_bear
                        .as_ref()
                        .map(|(_, r)| child.distance < r.distance)
                        .unwrap_or(true);
                    if replace {
                        best_bear = Some((mv, child));
                    }
                }
                Outcome::HuntersWin => {
                    let replace = best_hunter
                        .as_ref()
                        .map(|(_, r)| child.distance > r.distance)
                        .unwrap_or(true);
                    if replace {
                        best_hunter = Some((mv, child));
                    }
                }
            }
        }

        assemble_result(best_bear.or(best_hunter).expect("at least one move"))
    }
}

fn assemble_result((mv, child): (Move, SolveResult)) -> SolveResult {
    let mut pv = vec![mv.clone()];
    pv.extend(child.principal_variation);
    SolveResult {
        outcome: child.outcome,
        distance: child.distance + 1,
        best_move: Some(mv),
        principal_variation: pv,
    }
}

pub fn legal_hunter_moves(graph: &Graph, state: &GameState) -> Vec<Move> {
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
