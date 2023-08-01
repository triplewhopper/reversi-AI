use clap::Parser;
use std::thread;
use reversi::{
    agent::{Agent, OneStepLookaheadAgent, RandomAgent},
    board::{Action, Board, Player, Pos},
    command::SessionError,
    simulate,
};

#[derive(Parser, Debug)]
#[command(about)]
struct Cli {
    #[arg(
        short = 'H',
        default_value = "localhost",
        help = "host name",
        )]
    host: String,

    #[arg(
        short = 'p',
        default_value_t = 3000,
        help = "port number",
        value_parser = clap::value_parser!(u16).range(1..)
    )]
    port: u16,

    #[arg(long, default_value_t = 10000u32, help = "number of simulations", value_parser = clap::value_parser!(u32).range(100..))]
    n_simulations: u32,
    #[arg(short, default_value = "Anon.", help="player name")]
    name: String,
    #[arg(short, default_value_t = false, help = "verbose mode")]
    verbose: bool,
}




fn main() -> Result<(), SessionError>{
    let cli = Cli::parse();
    eprintln!("Connecting to {}:{}...", cli.host, cli.port);
    let stream = std::net::TcpStream::connect((cli.host, cli.port))?;
    use reversi::command::Session;
    // let black = reversi::agent::RandomAgent;
    
    let mut session = Session::new(cli.name);
    let handle = thread::spawn(move|| {
        let mut white = reversi::mcts_agent::MCTSAgent::new(cli.n_simulations, 1.4);
        let _ = session.launch(&mut white, &mut &stream).map_err(|e| {eprintln!("{}", e); e});
    });
    handle.join().expect("The thread being joined has panicked");
    return Ok(());
    // let mut nblackwins = 0;
    // let mut nwhitewins = 0;
    // let mut total_steps = 0usize;

    // for i in 1..=1000 {
    //     let board = Board::initial();

    //     let mut n_steps = 0 as usize;
    //     match simulate!(board, &black, &white, &mut n_steps) {
    //         Some(Player::Black) => nblackwins += 1,
    //         Some(Player::White) => nwhitewins += 1,
    //         None => (),
    //     }
    //     total_steps += n_steps;
    //     if i % 1 == 0 {
    //         let blackrate = nblackwins as f64 / (nblackwins + nwhitewins) as f64;
    //         let whiterate = nwhitewins as f64 / (nblackwins + nwhitewins) as f64;
    //         println!(
    //             "\r{:5}: Black: {:5} ({:.2}%), White: {:5} ({:.2}%) Ave. steps: {:.4}",
    //             i,
    //             nblackwins,
    //             blackrate * 100.0,
    //             nwhitewins,
    //             whiterate * 100.0,
    //             total_steps as f64 / i as f64
    //         );
    //     }
    // }
    // println!("Black wins: {}", nblackwins);
    // println!("White wins: {}", nwhitewins);
    
    Ok(())
}
