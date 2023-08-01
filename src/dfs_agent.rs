use crate::board::{Action, Board, Player, Pos};
use crate::agent::Agent;
pub struct DfsAgent {
    depth: usize,
}
impl DfsAgent {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
    fn dfs(&self, board: &Board, p: Player, depth: usize) -> i32 {
        if depth == 0 {
            return board.count(p) as i32;
        }
        let mut best_score = -1000;
        for Action{at, board} in board.valid_moves(p) {
            let score = -self.dfs(&board, p.opponent(), depth - 1);
            if score > best_score {
                best_score = score;
            }
        }
        best_score
    }
}
impl Agent for DfsAgent {
    fn opponent_move_callback(&mut self, _action: Option<Action>) {
        todo!()
    }
    fn select_move(&mut self, _board: &Board, _p: Player) -> Option<Action> {
        unimplemented!()
        // let mut best_action = None;
        // let mut best_score = -1000;
        // for Action{at, board} in board.valid_moves(p) {
        //     let score = self.dfs(&board, p, self.depth);
        //     if score > best_score {
        //         best_action = Some(action);
        //         best_score = score;
        //     }
        // }
        // best_action
    }
}