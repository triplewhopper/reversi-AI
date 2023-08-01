use crate::agent::Agent;
use crate::board::{Action, Board, Player, Pos};
use std::io::Write;
use std::net::TcpStream;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Milliseconds(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score<T>(T);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stat {
    name: String,
    score: Score<i32>,
    n_win: u32,
    n_lose: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    Start {
        bw: Player,
        opponent_name: String,
        remaining_time: Milliseconds,
    }, // player, opponent, remaining time
    End {
        result: Wl,
        my_score: Score<u32>,
        opponent_score: Score<u32>,
        reason: String,
    }, // win or lose, black score, white score, reason
    Move(Option<Pos>),
    Ack {
        remaining_time: Milliseconds,
    },
    Bye {
        stat: Vec<Stat>,
    },
}
impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Cmd::*;
        match self {
            Start {
                bw,
                opponent_name,
                remaining_time,
            } => write!(f, "START {:?} {} {}", bw, opponent_name, remaining_time.0),
            End {
                result,
                my_score,
                opponent_score,
                reason,
            } => write!(
                f,
                "END {:?} {} {} {}",
                result, my_score.0, opponent_score.0, reason
            ),
            Move(None) => write!(f, "MOVE PASS"),
            Move(Some(pos)) => write!(f, "MOVE {}", pos),
            Ack { remaining_time } => write!(f, "ACK {}", remaining_time.0),
            Bye { stat } => {
                write!(f, "BYE")?;
                for s in stat {
                    write!(f, " {} {} {} {}", s.name, s.score.0, s.n_win, s.n_lose)?;
                }
                Ok(())
            }
        }
    }
}
impl Cmd {
    fn start<S>(bw: Player, opponent_name: S, remaining_time: Milliseconds) -> Cmd
    where
        S: Into<String>,
    {
        let opponent_name = opponent_name.into();
        Cmd::Start {
            bw,
            opponent_name,
            remaining_time,
        }
    }
    fn end<S>(result: Wl, my_score: u32, opponent_score: u32, reason: S) -> Cmd
    where
        S: Into<String>,
    {
        Cmd::End {
            result,
            my_score: Score(my_score),
            opponent_score: Score(opponent_score),
            reason: reason.into(),
        }
    }
    fn ack(remaining_time: Milliseconds) -> Cmd {
        Cmd::Ack { remaining_time }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wl {
    Win,
    Lose,
    Tie,
}

#[derive(Error, Debug)]
pub enum CmdParseError {
    #[error("IO error:")]
    IoError(#[from] std::io::Error),
    #[error("ParseIntError:")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("ParsePosError:")]
    ParsePosError(#[from] crate::board::ParsePosError),
    #[error("Non-ASCII character: {0}")]
    NonAscii(String),
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("{0}: unexpected token {1}")]
    UnexpectedToken(&'static str, String),
    #[error("{0}: expected {1}, found {2}")]
    ExpectedFound(&'static str, &'static str, String),
    #[error("{0}: missing {1}")]
    Missing(&'static str, &'static str),
}

impl TryFrom<&str> for Cmd {
    type Error = CmdParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let tokens: Vec<_> = s.split_whitespace().collect();
        match &tokens[..] {
            [] => Err(CmdParseError::Missing("command", "command")),
            ["START"] => Err(CmdParseError::Missing("START", "WHITE or BLACK")),
            ["START", _] => Err(CmdParseError::Missing("START", "opponent name")),
            ["START", _, _] => Err(CmdParseError::Missing("START", "remaining time")),
            ["START", wb, opponent_name, remaining_time] => {
                use Player::*;
                let remaining_time = remaining_time.parse::<u32>().map_err(|e| {
                    CmdParseError::ExpectedFound("START", "nonnegative integer", e.to_string())
                })?;
                let remaining_time = Milliseconds(remaining_time);
                match *wb {
                    "WHITE" => Ok(Cmd::start(White, *opponent_name, remaining_time)),
                    "BLACK" => Ok(Cmd::start(Black, *opponent_name, remaining_time)),
                    _ => Err(CmdParseError::ExpectedFound(
                        "START",
                        "WHITE or BLACK",
                        s.to_string(),
                    )),
                }
            }
            ["START", _, _, _, s, ..] => {
                Err(CmdParseError::UnexpectedToken("START", s.to_string()))
            }
            ["END"] => Err(CmdParseError::Missing("END", "WIN or LOSE")),
            ["END", _] => Err(CmdParseError::Missing("END", "my score")),
            ["END", _, _] => Err(CmdParseError::Missing("END", "opponent score")),
            ["END", _, _, _] => Err(CmdParseError::Missing("END", "reason")),
            ["END", wl, n, m, reason] => {
                let n = n.parse::<u32>()?;
                let m = m.parse::<u32>()?;
                match *wl {
                    "WIN" => {
                        // assert!(n > m);
                        Ok(Cmd::end(Wl::Win, n, m, reason.to_string()))
                    }
                    "LOSE" => {
                        // assert!(n < m);
                        Ok(Cmd::end(Wl::Lose, n, m, reason.to_string()))
                    }
                    "TIE" => {
                        // assert_eq!(n, m);
                        Ok(Cmd::end(Wl::Tie, n, m, reason.to_string()))
                    }
                    _ => unreachable!(),
                }
            }
            ["END", _, _, _, _, s, ..] => Err(CmdParseError::UnexpectedToken("END", s.to_string())),
            ["MOVE"] => Err(CmdParseError::Missing("MOVE", "position (e.g. A1)")),
            ["MOVE", "PASS"] => Ok(Cmd::Move(None)),
            ["MOVE", pos] => {
                let pos: Pos = Pos::try_from(*pos)?;
                Ok(Cmd::Move(Some(pos)))
            }
            ["MOVE", _, s, ..] => Err(CmdParseError::UnexpectedToken("MOVE", s.to_string())),
            ["ACK"] => Err(CmdParseError::Missing(
                "ACK",
                "remaining time, in ms (e.g. 1000)",
            )),
            ["ACK", remaining_time] => {
                let remaining_time = remaining_time.parse::<u32>()?;
                Ok(Cmd::ack(Milliseconds(remaining_time)))
            }
            ["BYE", stat @ ..] => {
                let mut stat = stat;
                let mut vec = Vec::new();
                if stat.len() % 4 != 0 {
                    return Err(CmdParseError::ExpectedFound(
                        "BYE",
                        "multiple of 4 tokens",
                        s.to_string(),
                    ));
                }
                loop {
                    match stat {
                        [] => return Ok(Cmd::Bye { stat: vec }),
                        [_] | [_, _] | [_, _, _] => {
                            return Err(CmdParseError::ExpectedFound(
                                "BYE",
                                "multiple of 4 tokens",
                                s.to_string(),
                            ))
                        }
                        [name, score, n_win, n_lose, stat1 @ ..] => {
                            let score = score.parse::<i32>().map_err(|e| {
                                CmdParseError::ExpectedFound(
                                    "BYE",
                                    "integer",
                                    score.to_string(),
                                )
                            })?;
                            let n_win = n_win.parse::<u32>().map_err(|e| {
                                CmdParseError::ExpectedFound(
                                    "BYE",
                                    "n_win >= 0",
                                    n_win.to_string(),
                                )
                            })?;
                            let n_lose = n_lose.parse::<u32>().map_err(|e| {
                                CmdParseError::ExpectedFound(
                                    "BYE",
                                    "n_lose >= 0",
                                    n_lose.to_string(),
                                )
                            })?;
                            let score = Score(score);
                            vec.push(Stat {
                                name: name.to_string(),
                                score,
                                n_win,
                                n_lose,
                            });
                            stat = stat1;
                        }
                    }
                }
            }
            [s, ..] => Err(CmdParseError::UnknownCommand(s.to_string())),
        }
    }
}
impl TryFrom<String> for Cmd {
    type Error = <Cmd as TryFrom<&'static str>>::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Cmd::try_from(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_try_from_start() -> Result<(), <Cmd as TryFrom<&'static str>>::Error> {
        for (bw, bw1) in std::iter::zip(["WHITE", "BLACK"], [Player::White, Player::Black]) {
            for p1 in [
                "a",
                "a-c",
                ",,a",
                "Joe",
                "Adam",
                "BritishAirways",
                "🇨🇳",
                "😄",
            ] {
                for p2 in [
                    "a",
                    "a-c",
                    ",,a",
                    "Joe",
                    "Smith",
                    "BritishAirways",
                    "🇨🇳",
                    "😄",
                ] {
                    for t in ["0", "0000", "1", "4242", "1000", "1000000000"] {
                        let cmd = Cmd::try_from(format!("START {} {} {}", bw, p1, t))?;
                        assert_eq!(cmd, Cmd::start(bw1, p1, Milliseconds(t.parse::<u32>()?)));
                    }
                }
            }
        }
        Ok(())
    }
    #[test]
    fn test_try_from_end() -> Result<(), <Cmd as TryFrom<&'static str>>::Error> {
        assert_eq!(
            Cmd::try_from("END WIN 40 20 DOUBLE_PASS")?,
            Cmd::end(Wl::Win, 40, 20, "DOUBLE_PASS")
        );
        assert_eq!(
            Cmd::try_from("END LOSE 10 20 DOUBLE_PASS")?,
            Cmd::end(Wl::Lose, 10, 20, "DOUBLE_PASS")
        );
        Ok(())
    }
    #[test]
    fn test_try_from_move() -> Result<(), <Cmd as TryFrom<&'static str>>::Error> {
        for rank in ['1', '2', '3', '4', '5', '6', '7', '8'] {
            for file in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] {
                let pos = format!("{}{}", file, rank);
                assert_eq!(
                    Cmd::try_from(format!("MOVE {}", pos))?,
                    Cmd::Move(Some(Pos::try_from(pos.as_str())?))
                );
            }
        }
        Ok(())
    }

    #[test]
    fn test_try_from_ack() -> Result<(), <Cmd as TryFrom<&'static str>>::Error> {
        for t in ["0", "0000", "1", "4242", "1000", "1000000000"] {
            assert_eq!(
                Cmd::try_from(format!("ACK {}", t))?,
                Cmd::ack(Milliseconds(t.parse::<u32>()?))
            );
        }
        Ok(())
    }

    #[test]
    fn test_try_from_bye() -> Result<(), <Cmd as TryFrom<&'static str>>::Error> {
        assert_eq!(Cmd::try_from("BYE")?, Cmd::Bye { stat: Vec::new() });
        assert_eq!(
            Cmd::try_from("BYE Joe 10 20 30")?,
            Cmd::Bye {
                stat: vec![Stat {
                    name: "Joe".to_string(),
                    score: Score(10),
                    n_win: 20,
                    n_lose: 30,
                }]
            }
        );
        assert_eq!(
            Cmd::try_from("BYE Joe 10 20 30 Adam 20 10 30")?,
            Cmd::Bye {
                stat: vec![
                    Stat {
                        name: "Joe".to_string(),
                        score: Score(10),
                        n_win: 20,
                        n_lose: 30,
                    },
                    Stat {
                        name: "Adam".to_string(),
                        score: Score(20),
                        n_win: 10,
                        n_lose: 30,
                    },
                ]
            }
        );
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("IO error:")]
    IoError(#[from] std::io::Error),
    #[error("Cmd parse error: {0}")]
    CmdParseError(#[from] CmdParseError),
    #[error("Unexpected command: {0:?}")]
    UnexpectedCmd(Cmd),
    #[error("Missing event handlers for {0:?}")]
    MissingEventHandlersFor(Cmd),
}

pub struct Session {
    pub self_name: String,
    pub opponent_name: Option<String>,
    // pub on_open: Box<dyn FnOnce()>,
    // pub on_black_start: Box<dyn FnOnce() -> Action>,
    // pub on_white_start: Box<dyn FnOnce()>,
    // pub on_move: Box<dyn Fn(Pos) -> Option<Action>>,
    // pub on_ack: Box<dyn Fn(Milliseconds)>,
    // pub on_end: Option<Box<dyn FnOnce(Wl, Score, Score, String) + Sync>>,
    // pub on_bye: Option<Box<dyn FnOnce(Vec<Stat>) + Sync>>,
}

impl Session {
    pub fn new(
        self_name: impl Into<String>,
        // on_open: impl FnOnce() + 'static,
        // on_black_start: impl FnOnce() -> Pos + 'static,
        // on_move: impl Fn(Pos) -> Option<Pos> + 'static,
        // on_ack: impl Fn(Milliseconds)+ 'static) -> Self {
    ) -> Self {
        Self {
            self_name: self_name.into(),
            opponent_name: None,
            // on_open: Box::new(on_open),
            // on_black_start: Box::new(on_black_start),
            // on_move: Box::new(on_move),
            // on_ack: Box::new(on_ack),
            // on_end: None,
            // on_bye: None,
        }
    }

    // pub fn on_end<F>(mut self, f: impl FnOnce(Wl, Score, Score, String) + Sync + 'static) -> Self {
    //     self.on_end = Some(Box::new(f));
    //     self
    // }

    // pub fn on_bye<F>(mut self, f: impl FnOnce(Vec<Stat>) + Sync + 'static) -> Self {
    //     self.on_bye = Some(Box::new(f));
    //     self
    // }
    fn read_cmd(reader: &mut impl std::io::BufRead) -> Result<Cmd, SessionError> {
        let mut line = String::new();
        let nbytes = reader.read_line(&mut line)?;
        
        if nbytes == 0 {
            return Err(SessionError::IoError(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)));
        }
        let cmd = Cmd::try_from(line.as_str())?;
        Ok(cmd)
    }

    pub fn launch(&mut self, agent: &mut impl Agent, stream: &TcpStream) -> Result<(), SessionError> {
        let mut reader = std::io::BufReader::new(stream);
        let mut writer = std::io::BufWriter::new(stream);

        writeln!(writer, "OPEN {}", self.self_name)?;
        eprintln!("[{}] 发 OPEN {}", self.self_name, self.self_name);
        writer.flush()?;
        // (self.on_open)();
        agent.initialize();
        loop {
            let cmd = Session::read_cmd(&mut reader)?;
            eprintln!("[{}] 收 {}", self.self_name, cmd);

            let mut board: Board = Board::initial();

            match cmd {
                Cmd::Start {
                    bw,
                    opponent_name,
                    remaining_time,
                } => {
                    self.opponent_name = Some(opponent_name);
                    if bw == Player::Black {
                        
                        let action = agent.select_move(&board, bw).expect("no move");
                        board = action.board;
                        writeln!(writer, "MOVE {}", action.at)?;
                        writer.flush()?;
                        eprintln!("[{}] 发 MOVE {}", self.self_name, action.at);
                    } 

                    loop {
                        let cmd = Session::read_cmd(&mut reader)?;
                        eprintln!("[{}] 收 {}", self.self_name, cmd);
                        match cmd {
                            Cmd::Start { .. } | Cmd::Bye { .. } => {
                                return Err(SessionError::UnexpectedCmd(cmd));
                            }
                            Cmd::Move(pos) => {
                                if let Some(opponent_action) = pos.map(|pos| {
                                    board
                                        .valid_moves(bw.opponent())
                                        .find_map(
                                            |a| if a.at == pos { Some(a) } else { None },
                                        )
                                        .unwrap()
                                }) {
                                    board = opponent_action.board;
                                    agent.opponent_move_callback(Some(opponent_action));
                                } else {
                                    agent.opponent_move_callback(None);
                                }

                                if let Some(pos) = agent.select_move(&board, bw).map(|action| {
                                    board = action.board;
                                    action.at
                                }) {
                                    writeln!(writer, "MOVE {}", pos)?;
                                    writer.flush()?;
                                    eprintln!("[{}] 发 MOVE {}", self.self_name, pos);
                                } else {
                                    writeln!(writer, "MOVE PASS")?;
                                    writer.flush()?;
                                    eprintln!("[{}] 发 MOVE PASS", self.self_name);
                                }
                            }
                            Cmd::Ack { remaining_time } => {
                                // (self.on_ack)(remaining_time);
                            }
                            Cmd::End {
                                result,
                                my_score,
                                opponent_score,
                                reason,
                            } => {
                                agent.initialize();
                                // self.on_end
                                //     .map(|f| f(result, my_score, opponent_score, reason));
                                break;
                            }
                        }
                    }
                }
                Cmd::Bye { stat } => {
                    return Ok(());
                }
                _ => {
                    return Err(SessionError::UnexpectedCmd(cmd));
                }
            }
        }
    }
}
