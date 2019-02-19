#![feature(await_macro, async_await, futures_api)]
#![feature(try_blocks)]
#![feature(try_from)]
#![feature(duration_float)]

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
}

fn main() {
   pretty_env_logger::init();
   let opt = Opt::from_args();

   let (ite_tx, ite_rx) = mpsc::channel(); // Interface to Engine
   let (eti_tx, eti_rx) = mpsc::channel(); // Engine to Interface
   thread::spawn(move || {
      engine::start(ite_rx, eti_tx);
   });

   /*
   ite_tx.send(messages::InterfaceMessage::Go(6)).unwrap();
   let _ = eti_rx.recv().unwrap();

   return;
   */
   
   if opt.lichess {
      lichess::main_loop(ite_tx, eti_rx)
   } else {
      uci::main_loop(ite_tx, eti_rx)
   }
}
