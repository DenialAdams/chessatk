#![feature(catch_expr)]
#![feature(try_from)]

extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate rand;
extern crate reqwest;

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
    /// Username for a bot account on lichess. Turns Lichess mode on, and UCI will be disabled
    #[structopt(long = "lichess-username")]
    lichess_username: Option<String>
}

fn main() {
    let opt = Opt::from_args();

    let (ite_tx, ite_rx) = mpsc::channel(); // Interface to Engine
    let (eti_tx, eti_rx) = mpsc::channel(); // Engine to Interface
    thread::spawn(move|| {
        engine::start(ite_rx, eti_tx);
    });

    if let Some(_) = opt.lichess_username {
        lichess::main_loop(ite_tx, eti_rx)
    } else {
        uci::main_loop(ite_tx, eti_rx)
    }
}
