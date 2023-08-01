use core::panic;
use std::fmt::{Display, write};


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Player {
    Black = 0,
    White = 1,
}
impl Player {
    pub fn opponent(&self) -> Player {
        match *self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
    pub fn flip(&mut self) {
        *self = self.opponent();
    }
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Pos{
    A1=63, B1=62, C1=61, D1=60, E1=59, F1=58, G1=57, H1=56,
    A2=55, B2=54, C2=53, D2=52, E2=51, F2=50, G2=49, H2=48,
    A3=47, B3=46, C3=45, D3=44, E3=43, F3=42, G3=41, H3=40,
    A4=39, B4=38, C4=37, D4=36, E4=35, F4=34, G4=33, H4=32,
    A5=31, B5=30, C5=29, D5=28, E5=27, F5=26, G5=25, H5=24,
    A6=23, B6=22, C6=21, D6=20, E6=19, F6=18, G6=17, H6=16,
    A7=15, B7=14, C7=13, D7=12, E7=11, F7=10, G7=9,  H7=8,
    A8=7,  B8=6,  C8=5,  D8=4,  E8=3,  F8=2,  G8=1,  H8=0,
}


impl Pos {
    pub fn iter() -> std::slice::Iter<'static, Pos> {
        use Pos::*;
        static POSTIONS: [Pos; 64] = [
            A1, B1, C1, D1, E1, F1, G1, H1,
            A2, B2, C2, D2, E2, F2, G2, H2,
            A3, B3, C3, D3, E3, F3, G3, H3,
            A4, B4, C4, D4, E4, F4, G4, H4,
            A5, B5, C5, D5, E5, F5, G5, H5,
            A6, B6, C6, D6, E6, F6, G6, H6,
            A7, B7, C7, D7, E7, F7, G7, H7,
            A8, B8, C8, D8, E8, F8, G8, H8,
        ];
        POSTIONS.iter()
    }
    pub fn from_wthor_position(pos: &wthor::Position) -> Self{
        use wthor::Position;
        let &Position { rank, file } = pos;
        assert!(rank < 8);
        assert!(file < 8);
        let a = match rank {
            0 => 63,
            1 => 55,
            2 => 47,
            3 => 39,
            4 => 31,
            5 => 23,
            6 => 15,
            7 => 7,
            _ => panic!("Invalid rank: {}", rank),
        } - file;
        unsafe { std::mem::transmute(a) }
    }
}
impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(thiserror::Error, Clone, PartialEq, Eq, Hash, Debug)]
#[error("{0}: expected [A-H][1-8]")]
pub struct ParsePosError(String);

impl TryFrom<&str> for Pos {
    type Error = ParsePosError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut res = 0u8;
        if value.len() != 2 || !value.is_ascii() {
            return Err(ParsePosError(value.to_string()));
        }
        match value.get(1..2) {
            Some("1") => res += 63,
            Some("2") => res += 55,
            Some("3") => res += 47,
            Some("4") => res += 39,
            Some("5") => res += 31,
            Some("6") => res += 23,
            Some("7") => res += 15,
            Some("8") => res += 7,
            _ => return Err(ParsePosError(value.to_string())),
        };
        match value.get(0..1) {
            Some("A") => res -= 0,
            Some("B") => res -= 1,
            Some("C") => res -= 2,
            Some("D") => res -= 3,
            Some("E") => res -= 4,
            Some("F") => res -= 5,
            Some("G") => res -= 6,
            Some("H") => res -= 7,
            _ => return Err(ParsePosError(value.to_string())),
        };
        return unsafe { Ok(std::mem::transmute(res)) }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_from_wthor_position() {
        let mut pos_iter = Pos::iter();
        for rank in 0..8 {
            for file in 0..8 {
                let pos = wthor::Position { rank, file };
                let &pos2 = pos_iter.next().unwrap();
                assert_eq!(Pos::from_wthor_position(&pos), pos2, "rank={}, file={}", rank, file);
            }
        }
    }
    #[test]
    #[should_panic]
    fn test_from_wthor_position_panic() {
        Pos::from_wthor_position(&wthor::Position { rank: 8, file: 0 });
    }
}

fn north(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        A1 | B1 | C1 | D1 | E1 | F1 | G1 | H1 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 + 8)},
    }
}
fn south(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        A8 | B8 | C8 | D8 | E8 | F8 | G8 | H8 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 - 8)},
    }
}
fn east(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        H1 | H2 | H3 | H4 | H5 | H6 | H7 | H8 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 - 1)},
    }
}
fn west(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        A1 | A2 | A3 | A4 | A5 | A6 | A7 | A8 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 + 1)},
    }
}
fn northwest(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        A1 | B1 | C1 | D1 | E1 | F1 | G1 | H1 | A2 | A3 | A4 | A5 | A6 | A7 | A8 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 + 9)},
    }
}
fn southeast(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        H8 | G8 | F8 | E8 | D8 | C8 | B8 | A8 | H7 | H6 | H5 | H4 | H3 | H2 | H1 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 - 9)},
    }
}
fn northeast(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        H1 | G1 | F1 | E1 | D1 | C1 | B1 | A1 | H2 | H3 | H4 | H5 | H6 | H7 | H8 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 + 7)},
    }
}
fn southwest(xy: &Pos) -> Option<Pos> {
    use Pos::*;
    match xy {
        A8 | B8 | C8 | D8 | E8 | F8 | G8 | H8 | A7 | A6 | A5 | A4 | A3 | A2 | A1 => None,
        _ => unsafe {std::mem::transmute(*xy as u8 - 7)},
    }
}

enum CompassRose {
    North,
    South,
    East,
    West,
    Northwest,
    Southeast,
    Northeast,
    Southwest,
}

impl CompassRose {
    fn iter() -> std::slice::Iter<'static, CompassRose> {
        use CompassRose::*;
        static COMPASS_ROSE: [CompassRose; 8] = [
            North, South, East, West, Northwest, Southeast, Northeast, Southwest,
        ];
        COMPASS_ROSE.iter()
    }
    fn on(&self, xy: &Pos) -> std::iter::Successors<Pos, fn(&Pos) -> Option<Pos>> {
        use CompassRose::*;
        let f =  {
            match self {
                North => north,
                South => south,
                East => east,
                West => west,
                Northwest => northwest,
                Southeast => southeast,
                Northeast => northeast,
                Southwest => southwest,
            }
        };
        std::iter::successors(f(xy),f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board(u64, u64);
const MASK: u64 = 0x1;

trait Shift {
    fn shift(&self, d: &CompassRose) -> Self;
}
impl Shift for u64 {
    fn shift(&self, d: &CompassRose) -> Self {
        use CompassRose::*;
        match d {
            East => 
                (self >> 1) & 0x7f7f_7f7f_7f7f_7f7f,
            Southeast => 
                (self >> 9) & 0x007f_7f7f_7f7f_7f7f,
            South => 
                self >> 8, 
            Southwest => 
                (self >> 7) & 0x00fe_fefe_fefe_fefe,
            West => 
                (self << 1) & 0xfefe_fefe_fefe_fefe,
            Northwest => 
                (self << 9) & 0xfefe_fefe_fefe_fe00,
            North => 
                self << 8,
            Northeast =>
                (self << 7) & 0x7f7f_7f7f_7f7f_7f00,
        }
    }
}
fn flip_diag_a1_h8(mut x: u64) -> u64 {
    const K1: u64 = 0x5500_5500_5500_5500;
    const K2: u64 = 0x3333_0000_3333_0000;
    const K4: u64 = 0x0F0F_0F0F_0000_0000;

    let t = K4 & (x ^ (x << 28));
    x ^= t ^ (t >> 28);
    let t = K2 & (x ^ (x << 14));
    x ^= t ^ (t >> 14);
    let t = K1 & (x ^ (x << 7));
    x ^= t ^ (t >> 7);
    x
}

fn flip_diag_a8_h1(mut x: u64) -> u64 {
    const K1: u64 = 0xAA00_AA00_AA00_AA00;
    const K2: u64 = 0xCCCC_0000_CCCC_0000;
    const K4: u64 = 0xF0F0_F0F0_0F0F_0F0F;
    let t = x ^ (x << 36);
    x ^= K4 & (t ^ (x >> 36));
    let t = K2 & (x ^ (x << 18));
    x ^= t ^ (t >> 18);
    let t = K1 & (x ^ (x << 9));
    x ^= t ^ (t >> 9);
    x
}

fn rotate180(x: u64) -> u64 {
    const H1: u64 = 0x5555_5555_5555_5555;
    const H2: u64 = 0x3333_3333_3333_3333;
    const H4: u64 = 0x0F0F_0F0F_0F0F_0F0F;
    const V1: u64 = 0x00FF_00FF_00FF_00FF;
    const V2: u64 = 0x0000_FFFF_0000_FFFF;
    let x = ((x >> 1) & H1) | ((x & H1) << 1);
    let x = ((x >> 2) & H2) | ((x & H2) << 2);
    let x = ((x >> 4) & H4) | ((x & H4) << 4);
    let x = ((x >> 8) & V1) | ((x & V1) << 8);
    let x = ((x >> 16) & V2) | ((x & V2) << 16);
    let x = (x >> 32) | (x << 32);
    x
}
fn id<T>(x: T) -> T {x}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Action {
    pub at: Pos,
    pub board: Board,
}

impl Board {
    pub fn initial() -> Board {
        Board(
            0b_00000000_00000000_00000000_00001000_00010000_00000000_00000000_00000000,
            0b_00000000_00000000_00000000_00010000_00001000_00000000_00000000_00000000,
        )
    }
    pub fn from_disks(black: u64, white: u64) -> Board {
        assert!(black & white == 0);
        Board(black, white)
    }
    pub fn is_final(&self) -> bool {
        self.valid_moves_fast(Player::Black) == 0 && self.valid_moves_fast(Player::White) == 0
    }
    pub fn count(&self, p: Player) -> u32 {
        match p {
            Player::Black => self.0.count_ones(),
            Player::White => self.1.count_ones(),
        }
    }
    pub fn set(&self, xy: Pos, p: Player) -> Option<Board> {
        let mask = MASK << xy as u64;
        match p {
            Player::Black => match self.0 & mask {
                0 => Some(Board(self.0 | mask, self.1)),
                _ => None,
            },
            Player::White => match self.1 & mask {
                0 => Some(Board(self.0, self.1 | mask)),
                _ => None,
            },
        }
    }
    pub fn flip_discs(&self, disc_set: u64) -> Board {
        if (self.0 | self.1) & disc_set == disc_set {
            return Board(self.0 ^ disc_set, self.1 ^ disc_set);
        }
        panic!(
            "Invalid flip_discs: \n\
            self.0 = {:64b}s̄\n\
            self.1 = {:64b}\n\
            discs  = {:64b}\n", self.0, self.1, disc_set)
    }
    pub fn get(&self, xy: &Pos) -> Option<Player> {
        let mask = MASK << *xy as u64;
        if self.0 & mask != 0 {
            return Some(Player::Black);
        }
        if self.1 & mask != 0 {
            return Some(Player::White);
        }
        None
    }
    pub fn get_disks(&self, p: Player) -> u64 {
        match p {
            Player::Black => self.0,
            Player::White => self.1,
        }
    }
    pub fn flip_diag_a1_h8(&self) -> Board {
        Board(flip_diag_a1_h8(self.0), flip_diag_a1_h8(self.1))
    }
    pub fn flip_diag_a8_h1(&self) -> Board {
        Board(flip_diag_a8_h1(self.0), flip_diag_a8_h1(self.1))
    }
    pub fn rotate180(&self) -> Board {
        Board(rotate180(self.0), rotate180(self.1))
    }
    fn equivalent(&self, other: &Board) -> bool {
        if self == other {
            return true;
        }
        if self.flip_diag_a1_h8() == *other {
            return true;
        }
        if self.flip_diag_a8_h1() == *other {
            return true;
        }
        if self.rotate180() == *other {
            return true;
        }
        false
    }

    pub fn valid_moves_slow(&self, p: Player) -> impl Iterator<Item = Action> + '_ {
        Pos::iter().filter_map(move|xy| {
            if let None = self.get(xy) {
                let mut flip_candidate = 0u64;
                for dir in CompassRose::iter() {
                    let mut b = 0u64;
                    for xy in dir.on(xy) {
                        use Player::*;
                        match (p, self.get(&xy)) {
                            (_, None) => break,
                            (Black, Some(Black)) | (White, Some(White)) => {
                                flip_candidate |= b;
                                break;
                            },
                            (Black, Some(White)) | (White, Some(Black)) => {
                                b |= MASK << xy as u64;
                            }
                        }
                    }
                }
                if flip_candidate != 0 {
                    let res = Some(Action{ at: *xy, board: self.set(*xy, p).unwrap().flip_discs(flip_candidate)});
                    return res;
                }
            }
            return None;
        })
    }
    pub fn place_at_unchecked(&self, p: Player, i: Pos) -> Board {
            let my_disks = self.get_disks(p) | (MASK << i as u64);
            let opp_disks = self.get_disks(p.opponent());
            let mut captured_disks = 0u64;
            for d in CompassRose::iter() {
                let x = (MASK << i as u64).shift(d) & opp_disks;
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                if x.shift(d) & my_disks != 0 {
                    captured_disks |= x; 
                }
            }
            if p == Player::White {
                return Board(opp_disks ^ captured_disks, my_disks ^ captured_disks);
            }
            return Board(my_disks ^ captured_disks, opp_disks ^ captured_disks);
    }
    pub fn valid_moves(&self, p: Player) -> impl Iterator<Item = Action> + '_{
        let empty = !(self.0 | self.1);
        let mut moves = 0u64;
        let my_disks = self.get_disks(p);
        let opp_disks = self.get_disks(p.opponent());
        for d in CompassRose::iter() {
            let x = my_disks.shift(d) & opp_disks;
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            moves |= x.shift(d) & empty;
        }
        let f = match p {
            Player::Black=> |b: Board| b,
            Player::White=> |b: Board| Board(b.1, b.0),
        };
        let res = std::iter::from_fn(move|| {
            if moves == 0 {
                return None;
            }
            let i = moves & (!moves + 1);
            moves ^= i;
            let my_disks = my_disks | i;
            let mut captured_disks = 0u64;
            for d in CompassRose::iter() {
                let x = i.shift(d) & opp_disks;
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                let x = x | (x.shift(d) & opp_disks);
                if x.shift(d) & my_disks != 0 {
                    captured_disks |= x; 
                }
            }
            let pos = unsafe {
                std::mem::transmute(i.trailing_zeros() as u8)
            };
            return Some (Action{ at: pos, board: f(Board(my_disks ^ captured_disks, opp_disks ^ captured_disks))});
        });
        // let res = Pos::iter().filter_map(move|i| {
        //     if moves & (MASK << *i as u64) != 0 {
        //         let my_disks = my_disks | (MASK << *i as u64);
        //         let mut captured_disks = 0u64;
        //         for d in CompassRose::iter() {
        //             let x = (MASK << *i as u64).shift(d) & opp_disks;
        //             let x = x | (x.shift(d) & opp_disks);
        //             let x = x | (x.shift(d) & opp_disks);
        //             let x = x | (x.shift(d) & opp_disks);
        //             let x = x | (x.shift(d) & opp_disks);
        //             let x = x | (x.shift(d) & opp_disks);
        //             if x.shift(d) & my_disks != 0 {
        //                 captured_disks |= x; 
        //             }
        //         }
        //         return Some (Action{ at: *i, board: f(Board(my_disks ^ captured_disks, opp_disks ^ captured_disks))});
        //     }
        //     return None
        // });
        res
    }    
    pub fn valid_moves_fast(&self, p: Player) -> u64 {
        let empty = !(self.0 | self.1);
        let mut moves = 0u64;
        let my_disks = self.get_disks(p);
        let opp_disks = self.get_disks(p.opponent());
        for d in CompassRose::iter() {
            let x = my_disks.shift(d) & opp_disks;
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            let x = x | (x.shift(d) & opp_disks);
            moves |= x.shift(d) & empty;
        }
        moves
    }
}

// (Player, Board, Result<Pos, bool>) means (whose turn, board, last move (Err(true) means passed, Err(false) means initial))))))
pub fn boards_from<'a>(s0: Board, mut positions: impl Iterator<Item = Pos> + 'a) -> impl Iterator<Item = (Player, Board, Result<Pos, bool>)> + 'a {
    return std::iter::successors(Some((Player::Black, s0, Err(false))), move |&(p, b, last)| {
        let o = p.opponent();
        match b.valid_moves_fast(p) {
            0 => {
                if last == Err(true) { // both players passed
                    assert_eq!(None, positions.next()); // no more positions
                    return None;
                }
                return Some((o, b, Err(true)));
            },
            moves => {
                let Some(pos) = positions.next() else {
                    return None;
                };
                assert_ne!(0, moves & (MASK << pos as u64), "No valid move at {:?}, when {:?}'turn, board=\n{:?}", pos, p, b); 
                let res = Some((o, b.place_at_unchecked(p, pos), Ok(pos)));
                return res;
            }
        }
    });
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "\n  ABCDEFGH\n")?;
        for i in 0..8 {
            write!(f, "{} ", i + 1)?;
            for j in 0..8 {
                let mask = MASK << (63 - i * 8 - j);
                match (self.0 & mask, self.1 & mask) {
                    (0, 0) => write!(f, " ")?,
                    (0, _) => write!(f, "●")?,
                    (_, 0) => write!(f, "○")?,
                    _ => write!(f, "?")?,
                };
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
