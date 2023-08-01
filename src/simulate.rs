use crate::agent::Agent;
use crate::board::{Board, Player, Action};

pub fn simulate<'a>(
    board: Board,
    black: &mut impl Agent,
    white: &mut impl Agent,
    n_steps: &mut usize,
    call_back: &mut impl FnMut(Board, Player, Option<Action>),
) -> Option<Player> {
    let mut board = board;
    let mut p = Player::Black;
    let mut passed = false;
    *n_steps = 0;
    loop {
        if let Some(action) = match p {
            Player::Black => black.select_move(&board, p),
            Player::White => white.select_move(&board, p),
        } {
            // println!("{:?}: {:?}", p, action.at);
            // println!("{:?}", action.board);
            call_back(board, p, Some(action));
            board = action.board;
            *n_steps += 1;
            passed = false;
        } else {
            call_back(board, p, None);
            if passed {
                break;
            }
            passed = true;
        }
        p.flip();
    }
    // println!();
    match board.count(Player::Black).cmp(&board.count(Player::White)) {
        std::cmp::Ordering::Greater => Some(Player::Black),
        std::cmp::Ordering::Less => Some(Player::White),
        std::cmp::Ordering::Equal => None,
    }
}

#[macro_export]
macro_rules! simulate {
    ($board:expr, $black:expr, $white:expr, $n_steps:expr, $call_back:expr) => {
        crate::simulate::simulate($board, $black, $white, $n_steps, $call_back)
    };
    ($board:expr, $black:expr, $white:expr, $n_steps:expr) => {
        crate::simulate::simulate($board, $black, $white, $n_steps, &mut |_, _, _| {})
    };
    
}
