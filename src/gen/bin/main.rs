use reversi::agent::{bfs, Agent, OneStepLookaheadAgent, RandomAgent};
use reversi::board::{Board, Player, Pos, Action};
use reversi::simulate;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use wthor;
use rand::seq::{SliceRandom, IteratorRandom};
use wthor::{parse, WtbFile, WthorError};

fn from_wtb_file(year: i32) -> std::io::Result<Vec<wthor::Game>> {
    let filename = format!("../WTH_2001-2015/WTH_{}.wtb", year);
    let contents = std::fs::read(&filename)?;
    let wtb = wthor::parse(&contents).expect("Failed to parse WTH file");
    let games = wtb.games.expect("No games in WTH file");
    Ok(games)
}

fn gen_random_self_play<'a, 'b, 'c>(n: usize, dest: &str) -> std::io::Result<()> {
    let mut agent = RandomAgent;
    let mut agent2 = RandomAgent;
    let mut writer = BufWriter::new(std::fs::File::create(dest)?);
    writeln!(
        writer,
        "prev_self,prev_opponent,move,valid_moves,self,opponent"
    )?;
    for _ in 0..n {
        let board = Board::initial();
        let mut f = |b: Board, p: Player, a: Option<Action>| {
            a.map(|a| {
                writeln!(
                    writer,
                    "{},{},{},{},{},{}",
                    b.get_disks(p),
                    b.get_disks(p.opponent()),
                    a.at as u8,
                    b.valid_moves_fast(p),
                    a.board.get_disks(p),
                    a.board.get_disks(p.opponent())
                ).unwrap()
            });
        };
        let mut n_steps = 0 as usize;
        let winner = simulate!(board, &mut agent, &mut agent2, &mut n_steps, &mut f);
    }
    Ok(())
}

fn gen_boards_from_games(games: Vec<wthor::Game>, dest: &str) -> std::io::Result<()> {
    let mut writer = BufWriter::new(std::fs::File::create(dest)?);
    write!(
        writer,
        "prev_self,prev_opponent,move,valid_moves,self,opponent\n"
    )?;
    for game in games {
        let mut states = reversi::board::boards_from(
            Board::initial(),
            game.moves.iter().map(Pos::from_wthor_position),
        );
        let mut prev = states.next().unwrap();
        for (player, board, last_pos) in states {
            write!(writer, "{},", prev.1.get_disks(prev.0))?;
            write!(writer, "{},", prev.1.get_disks(prev.0.opponent()))?;
            match last_pos {
                Ok(last_pos) => {
                    write!(writer, "{},", last_pos as u64)?;
                }
                Err(true) => {
                    // passed
                    writer.write(b"64,")?;
                }
                Err(false) => {
                    panic!("this should not happen");
                }
            }
            // write_disks!(prev.1.valid_moves_fast(prev.0));
            // writer.write(b",")?;
            write!(writer, "{},", prev.1.valid_moves_fast(prev.0))?;
            // write!(writer, "{},", player as u8)?;
            write!(writer, "{},", board.get_disks(prev.0))?;
            write!(writer, "{}", board.get_disks(prev.0.opponent()))?;
            // write_disks!(board.get_disks(Player::Black));
            // writer.write(b",")?;
            // write_disks!(board.get_disks(Player::White));
            writer.write(b"\n")?;
            prev = (player, board, last_pos);
        }
        // writeln!(writer)?;
    }
    Ok(())
}

fn generate_action_for_random_board() -> std::io::Result<()>{
    use rand::Rng;
    let filename = "random_boards.csv";
    let mut writer = BufWriter::new(std::fs::File::create(filename)?);
    write!(
        writer,
        "prev_self,prev_opponent,move,valid_moves,self,opponent\n"
    )?;

    let mut rng = rand::thread_rng();
    let power2s: Vec<_> = (0..64).map(|x| 1 << x).collect();
    for _ in 0..1000 {
        for i in 0..64 {
            for j in 0..64 - i {
                let bw: Vec<_> = power2s.choose_multiple(&mut rng, i + j).collect();
                let tmp = bw.iter().fold(0u64, |acc, &x| (acc | x));
                let w = bw.choose_multiple(&mut rng, i).fold(0, |acc, &x| (acc | x));
                let b = tmp ^ w;
                write!(writer, "{},", b)?;
                write!(writer, "{},", w)?;
                let board = Board::from_disks(b, w);
                let moves = 0xffff_ffff_ffff_ffffu64 ^ (b | w);
                // let moves = board.valid_moves_fast(Player::Black);
                match board.valid_moves(Player::Black).choose(&mut rng) {
                    Some(action) => {
                        write!(writer, "{},", action.at as u64)?;
                        write!(writer, "{},", moves)?;
                        write!(writer, "{},", action.board.get_disks(Player::Black))?;
                        write!(writer, "{}", action.board.get_disks(Player::White))?;
                        writer.write(b"\n")?;
                    }
                    None => {
                        write!(writer, "64,")?;
                        write!(writer, "{},", moves)?;
                        write!(writer, "{},", board.get_disks(Player::Black))?;
                        write!(writer, "{}", board.get_disks(Player::White))?;
                        writer.write(b"\n")?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_game(games: &wthor::Game) {
    let wthor::Game { moves, .. } = games;
    assert!(moves.len() <= 60);
    assert!(moves.first().map(Pos::from_wthor_position) == Some(Pos::F5));

    let mut board = Board::initial();
    let mut p = Player::Black;
    for (_i, pos) in moves.iter().enumerate() {
        let m = Pos::from_wthor_position(pos);
        if board.valid_moves_fast(p) == 0 {
            p.flip();
        }
        let valid_moves: Vec<_> = board.valid_moves(p).collect();
        let valid_moves_slow: Vec<_> = board.valid_moves_slow(p).collect();
        assert_eq!(valid_moves, valid_moves_slow);
        assert!(!valid_moves.is_empty());
        let Some(action) = valid_moves.iter().find(|a| a.at == m) else {
            // eprintln!(
            //     "i={i}/{}; pos={pos:?}; m={m:?}, player={p:?}; board=\n{board:?}",
            //     moves.len()
            // );
            // eprintln!("moves={}", string_from_moves(moves));
            // for a in valid_moves {
            //     eprintln!("valid move: {:?}", a.at);
            // }
            panic!("No valid move at {:?}", m);
        };
        board = action.board;
        p.flip();
    }
    assert!(board.is_final());
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;
    macro_rules! test_wtb_years {
        ($($year:expr),*) => {
        $(
            paste! {
                #[test]
                fn [< test_wtb_ $year >]() {
                    let a = from_wtb_file($year).unwrap();
                    for game in a {
                        check_game(&game);
                    }
                }
            }
        )*
        }
    }
    test_wtb_years!(
        2001, 2002, 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011, 2012, 2013, 2014, 2015
    );
}

fn main() -> std::io::Result<()> {
    gen_random_self_play(20000, "self_play.csv")?;
    Ok(())
}
