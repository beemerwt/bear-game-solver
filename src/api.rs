use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{
    boards::bear_game_board::BEAR_GAME_BOARD,
    game::{GameState, Move, Outcome, SideToMove},
    graph::Graph,
    solver::{Solver, apply_move, legal_bear_moves, legal_hunter_moves},
};

#[derive(Clone)]
pub struct AppState {
    pub graph: Arc<Graph>,
    pub solver: Arc<Mutex<Solver<'static>>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGameState {
    pub bear: String,
    pub hunters: [String; 3],
    pub side_to_move: SideToMove,
    pub hunter_turns_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum ApiMove {
    Hunter {
        hunter_index: usize,
        from: String,
        to: String,
    },
    Bear {
        from: String,
        to: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardResponse {
    name: String,
    turn_limit: u32,
    nodes: Vec<NodeResponse>,
    edges: Vec<[String; 2]>,
    start: ApiGameState,
}
#[derive(Serialize)]
pub struct NodeResponse {
    id: String,
    x: i32,
    y: i32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalMovesResponse {
    legal_moves: Vec<ApiMove>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponse {
    outcome: Outcome,
    distance: u32,
    best_move: Option<ApiMove>,
    winning_moves: Vec<ApiMove>,
    bear_refutations: Vec<ApiMove>,
    explanation: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyMoveRequest {
    state: ApiGameState,
    #[serde(alias = "mv")]
    move_payload: ApiMove,
}
#[derive(Serialize)]
pub struct ApplyMoveResponse {
    state: ApiGameState,
}

pub async fn board(State(state): State<AppState>) -> impl IntoResponse {
    let nodes = state
        .graph
        .nodes
        .values()
        .map(|n| NodeResponse {
            id: n.id.to_string(),
            x: n.x,
            y: n.y,
        })
        .collect();
    let mut edges = Vec::new();
    for (from, tos) in &state.graph.adjacency {
        for to in tos {
            if from < to {
                edges.push([from.to_string(), to.to_string()]);
            }
        }
    }
    let start = ApiGameState {
        bear: BEAR_GAME_BOARD.start.bear.to_string(),
        hunters: BEAR_GAME_BOARD.start.hunters.map(|h| h.to_string()),
        side_to_move: BEAR_GAME_BOARD.start.side_to_move,
        hunter_turns_used: 0,
    };
    Json(BoardResponse {
        name: state.graph.name.to_string(),
        turn_limit: state.graph.turn_limit,
        nodes,
        edges,
        start,
    })
}

pub async fn legal_moves(
    State(state): State<AppState>,
    Json(req): Json<ApiGameState>,
) -> impl IntoResponse {
    match to_internal_state(&state.graph, req) {
        Ok(gs) => Json(LegalMovesResponse {
            legal_moves: moves_for_state(&state.graph, &gs)
                .into_iter()
                .map(to_api_move)
                .collect(),
        })
        .into_response(),
        Err(e) => bad(e),
    }
}

pub async fn evaluate(
    State(state): State<AppState>,
    Json(req): Json<ApiGameState>,
) -> impl IntoResponse {
    let gs = match to_internal_state(&state.graph, req) {
        Ok(v) => v,
        Err(e) => return bad(e),
    };
    let mut solver = state.solver.lock().expect("lock");
    let eval = solver.evaluate_state(gs.clone());
    let legal = moves_for_state(&state.graph, &gs);
    let best_move = match gs.side_to_move {
        SideToMove::Hunters => eval
            .winning_hunter_moves
            .first()
            .cloned()
            .or_else(|| legal.first().cloned()),
        SideToMove::Bear => eval.refuting_bear_moves.first().cloned().or_else(|| {
            legal
                .iter()
                .max_by_key(|m| solver.evaluate_state(apply_move(&gs, m)).distance)
                .cloned()
        }),
    };
    Json(EvaluateResponse {
        outcome: eval.outcome,
        distance: eval.distance,
        best_move: best_move.map(to_api_move),
        winning_moves: eval
            .winning_hunter_moves
            .into_iter()
            .map(to_api_move)
            .collect(),
        bear_refutations: eval
            .refuting_bear_moves
            .into_iter()
            .map(to_api_move)
            .collect(),
        explanation: format!("State is {:?} under perfect play.", eval.outcome),
    })
    .into_response()
}

pub async fn apply_move_endpoint(
    State(state): State<AppState>,
    Json(req): Json<ApplyMoveRequest>,
) -> impl IntoResponse {
    let gs = match to_internal_state(&state.graph, req.state) {
        Ok(v) => v,
        Err(e) => return bad(e),
    };
    let mv = match to_internal_move(&state.graph, &gs, req.move_payload) {
        Ok(v) => v,
        Err(e) => return bad(e),
    };
    if !moves_for_state(&state.graph, &gs).contains(&mv) {
        return bad("illegal move for current state".into());
    }
    Json(ApplyMoveResponse {
        state: to_api_state(&apply_move(&gs, &mv)),
    })
    .into_response()
}

fn moves_for_state(graph: &Graph, state: &GameState) -> Vec<Move> {
    match state.side_to_move {
        SideToMove::Hunters => legal_hunter_moves(graph, state),
        SideToMove::Bear => legal_bear_moves(graph, state),
    }
}
fn bad(msg: String) -> axum::response::Response {
    (StatusCode::BAD_REQUEST, Json(ApiError { error: msg })).into_response()
}
fn map_node(graph: &Graph, id: &str) -> Result<&'static str, String> {
    graph
        .nodes
        .keys()
        .copied()
        .find(|n| *n == id)
        .ok_or_else(|| format!("unknown node id: {id}"))
}
fn to_internal_state(graph: &Graph, s: ApiGameState) -> Result<GameState, String> {
    let bear = map_node(graph, &s.bear)?;
    let mut seen = HashSet::new();
    let mut hunters = [""; 3];
    for (i, h) in s.hunters.iter().enumerate() {
        let v = map_node(graph, h)?;
        if !seen.insert(v) {
            return Err("hunters must occupy 3 distinct nodes".into());
        }
        hunters[i] = v;
    }
    if hunters.contains(&bear) {
        return Err("bear cannot occupy a hunter node".into());
    }
    if s.hunter_turns_used > graph.turn_limit {
        return Err(format!("hunterTurnsUsed must be <= {}", graph.turn_limit));
    }
    hunters.sort_unstable();
    Ok(GameState {
        bear,
        hunters,
        side_to_move: s.side_to_move,
        hunter_turns_used: s.hunter_turns_used,
    })
}
fn to_internal_move(graph: &Graph, state: &GameState, mv: ApiMove) -> Result<Move, String> {
    Ok(match mv {
        ApiMove::Hunter {
            hunter_index,
            from,
            to,
        } => {
            if state.side_to_move != SideToMove::Hunters {
                return Err("not hunters turn".into());
            }
            Move::Hunter {
                hunter_index,
                from: map_node(graph, &from)?,
                to: map_node(graph, &to)?,
            }
        }
        ApiMove::Bear { from, to } => {
            if state.side_to_move != SideToMove::Bear {
                return Err("not bear turn".into());
            }
            Move::Bear {
                from: map_node(graph, &from)?,
                to: map_node(graph, &to)?,
            }
        }
    })
}
fn to_api_move(mv: Move) -> ApiMove {
    match mv {
        Move::Hunter {
            hunter_index,
            from,
            to,
        } => ApiMove::Hunter {
            hunter_index,
            from: from.to_string(),
            to: to.to_string(),
        },
        Move::Bear { from, to } => ApiMove::Bear {
            from: from.to_string(),
            to: to.to_string(),
        },
    }
}
fn to_api_state(st: &GameState) -> ApiGameState {
    ApiGameState {
        bear: st.bear.to_string(),
        hunters: st.hunters.map(|h| h.to_string()),
        side_to_move: st.side_to_move,
        hunter_turns_used: st.hunter_turns_used,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{boards::bear_game_board::BEAR_GAME_BOARD, graph::Graph};
    fn graph() -> Graph {
        Graph::from_board_definition(&BEAR_GAME_BOARD).unwrap()
    }
    #[test]
    fn state_conversion() {
        let g = graph();
        let api = ApiGameState {
            bear: "top".into(),
            hunters: ["bottom_left".into(), "bottom".into(), "bottom_right".into()],
            side_to_move: SideToMove::Hunters,
            hunter_turns_used: 0,
        };
        let st = to_internal_state(&g, api).unwrap();
        assert_eq!(st.side_to_move, SideToMove::Hunters);
    }
    #[test]
    fn invalid_state_rejected() {
        let g = graph();
        let api = ApiGameState {
            bear: "bad".into(),
            hunters: ["bottom_left".into(), "bottom".into(), "bottom_right".into()],
            side_to_move: SideToMove::Hunters,
            hunter_turns_used: 0,
        };
        assert!(to_internal_state(&g, api).is_err());
    }
    #[test]
    fn illegal_move_rejected() {
        let g = graph();
        let st = to_internal_state(
            &g,
            ApiGameState {
                bear: "top".into(),
                hunters: ["bottom_left".into(), "bottom".into(), "bottom_right".into()],
                side_to_move: SideToMove::Hunters,
                hunter_turns_used: 0,
            },
        )
        .unwrap();
        let mv = to_internal_move(
            &g,
            &st,
            ApiMove::Hunter {
                hunter_index: 0,
                from: "top".into(),
                to: "center".into(),
            },
        )
        .unwrap();
        assert!(!moves_for_state(&g, &st).contains(&mv));
    }
    #[test]
    fn hunter_move_increments_and_switches() {
        let g = graph();
        let st = to_internal_state(
            &g,
            ApiGameState {
                bear: "top".into(),
                hunters: ["bottom_left".into(), "bottom".into(), "bottom_right".into()],
                side_to_move: SideToMove::Hunters,
                hunter_turns_used: 0,
            },
        )
        .unwrap();
        let mv = legal_hunter_moves(&g, &st)[0].clone();
        let next = apply_move(&st, &mv);
        assert_eq!(next.hunter_turns_used, 1);
        assert_eq!(next.side_to_move, SideToMove::Bear);
    }
    #[test]
    fn bear_move_no_increment_switches() {
        let g = graph();
        let st = GameState {
            bear: "top",
            hunters: ["bottom", "bottom_left", "bottom_right"],
            side_to_move: SideToMove::Bear,
            hunter_turns_used: 3,
        };
        let mv = legal_bear_moves(&g, &st)[0].clone();
        let next = apply_move(&st, &mv);
        assert_eq!(next.hunter_turns_used, 3);
        assert_eq!(next.side_to_move, SideToMove::Hunters);
    }
    #[test]
    fn start_eval_consistent() {
        let g = graph();
        let start = GameState::from_start(&BEAR_GAME_BOARD.start);
        let mut s1 = Solver::new(&g);
        let mut s2 = Solver::new(&g);
        assert_eq!(
            s1.evaluate_state(start.clone()).outcome,
            s2.evaluate_state(start).outcome
        );
    }
}
