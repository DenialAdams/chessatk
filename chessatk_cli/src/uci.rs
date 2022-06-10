use chessatk_lib::messages::{EngineMessage, InterfaceMessage};
use std::sync::mpsc;

pub fn main_loop(_sender: mpsc::Sender<InterfaceMessage>, _receiver: mpsc::Receiver<EngineMessage>) {
   unimplemented!()
}
