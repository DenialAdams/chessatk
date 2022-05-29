#![feature(try_blocks)]

mod board;
mod engine;
mod lichess;
mod messages;
mod uci;

use std::sync::mpsc;
use std::thread;
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
}

fn main() {
   pretty_env_logger::init();
   let opt = Opt::from_args();

   let (ite_tx, ite_rx) = mpsc::channel(); // Interface to Engine
   let (eti_tx, eti_rx) = mpsc::channel(); // Engine to Interface
   thread::spawn(move || {
      engine::start(ite_rx, eti_tx);
   });

   if opt.profiling {
      let state = crate::board::State::from_fen("1Bb3BN/R2Pk2r/1Q5B/4q2R/2bN4/4Q1BK/1p6/1bq1R1rb w - - 0 1").unwrap();
      ite_tx.send(messages::InterfaceMessage::SetState(state)).unwrap();
      ite_tx.send(messages::InterfaceMessage::GoDepth(5)).unwrap();
      let _ = eti_rx.recv().unwrap();
      return;
   }

   if opt.lichess {
      lichess::main_loop(ite_tx, eti_rx)
   } else {
      uci::main_loop(ite_tx, eti_rx)
   }
}
