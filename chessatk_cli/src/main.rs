#![feature(let_chains)]

mod lichess;
mod uci;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

/// A simple chess engine
#[derive(StructOpt, Debug)]
#[structopt(name = "chessatk")]
struct Opt {
   /// Turns Lichess mode on, and UCI will be disabled
   #[structopt(short = "l", long = "lichess")]
   lichess: bool,
   /// Crude profiling mode
   #[structopt(short = "p", long = "profile")]
   profiling: bool,
   /// monte carlo tree search w/ ucb1 instead of negamax
   #[structopt(long = "mcts")]
   mcts: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
   pretty_env_logger::init();
   let opt = Opt::from_args();

   let (ite_tx, ite_rx) = mpsc::channel(); // Interface to Engine
   let (eti_tx, eti_rx) = mpsc::channel(); // Engine to Interface

   if opt.mcts {
      thread::spawn(move || {
         chessatk_lib::mcts::start(ite_rx, eti_tx);
      });
   } else {
      thread::spawn(move || {
         chessatk_lib::engine::start(ite_rx, eti_tx);
      });
   }

   if opt.profiling {
      let state =
         chessatk_lib::board::State::from_fen("r1bq1rk1/ppp1p1bp/2np1np1/5p2/2PP4/2N2NP1/PP2PPBP/R1BQ1RK1 w - - 2 8")
            .unwrap();
      ite_tx
         .send(chessatk_lib::messages::InterfaceMessage::SetState(state))
         .unwrap();
      ite_tx
         .send(chessatk_lib::messages::InterfaceMessage::GoTime(Duration::from_secs(
            30,
         )))
         .unwrap();
      let _ = eti_rx.recv().unwrap();

      return;
   }

   if opt.lichess {
      lichess::main_loop(ite_tx, eti_rx).await;
   } else {
      uci::main_loop(ite_tx, eti_rx);
   }
}
