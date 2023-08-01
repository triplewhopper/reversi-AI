
use crate::board::{Action, Board, Player, Pos};
use std::collections::{HashSet, VecDeque};
use rand::seq::IteratorRandom;

pub trait Agent {
    fn initialize(&mut self) {}
    fn opponent_move_callback(&mut self, _action: Option<Action>) {}
    fn select_move(&mut self, board: &Board, p: Player) -> Option<Action>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomAgent;
impl Agent for RandomAgent {
    fn opponent_move_callback(&mut self, _action: Option<Action>) {}
    fn select_move(&mut self, board: &Board, p: Player) -> Option<Action> {
        board.valid_moves(p).choose(&mut rand::thread_rng())
    }
}
pub struct OneStepLookaheadAgent;
impl Agent for OneStepLookaheadAgent {
    fn opponent_move_callback(&mut self, _action: Option<Action>) {}
    fn select_move(&mut self, board: &Board, p: Player) -> Option<Action> {
        // if corner is available, take it; otherwise, take move randomly
        let moves: Vec<_> = board.valid_moves(p).collect();
        if let Some(corner) = moves.iter().find(|m| match m.at {
            Pos::A1 | Pos::A8 | Pos::H1 | Pos::H8 => true,
            _ => false,
        }) {
            return Some(*corner);
        }

        if let Some(action) = moves
            .iter()
            .filter(|m| m.board.valid_moves_fast(p.opponent()) & 0x8100000000000081 == 0)
            .choose(&mut rand::thread_rng())
        {
            return Some(*action);
        }
        return moves.into_iter().choose(&mut rand::thread_rng());
    }
}

pub fn bfs(board: &Board, p: Player, depth: usize) -> HashSet<(Player, Board)> {
    let mut vis = HashSet::from([(p, board.clone())]);
    let mut q = VecDeque::from([(depth, p, board.clone())]);
    while let Some((d, p, b)) = q.pop_front() {
        let actions: Vec<_> = b.valid_moves(p).collect();
        let o = p.opponent();
        if !actions.is_empty() {
            for action in b.valid_moves(p) {
                if !vis.contains(&(o, action.board))
                    && !vis.contains(&(o, action.board.flip_diag_a1_h8()))
                    && !vis.contains(&(o, action.board.flip_diag_a8_h1()))
                    && !vis.contains(&(o, action.board.rotate180()))
                {
                    vis.insert((o, action.board));
                    if d > 1 {
                        q.push_back((d - 1, o, action.board));
                    }
                }
            }
        } else {
            if !vis.contains(&(o, b))
                && !vis.contains(&(o, b.flip_diag_a1_h8()))
                && !vis.contains(&(o, b.flip_diag_a8_h1()))
                && !vis.contains(&(o, b.rotate180()))
            {
                vis.insert((o, b));
                if d > 1 {
                    q.push_back((d - 1, o, b));
                }
            }
        }
    }
    vis
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bfs() {
        let board = Board::initial();
        let vis = bfs(&board, Player::Black, 1);
        assert_eq!(vis.len(), 1 + 1);
        let vis = bfs(&board, Player::Black, 5);
        assert_eq!(vis.len(), 1 + 400);
        let vis = bfs(&board, Player::Black, 7);
        assert_eq!(vis.len(), 12832);
        let vis = bfs(&board, Player::Black, 10);
        assert_eq!(vis.len(), 3496888);
    }
}