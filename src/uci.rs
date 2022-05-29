use crate::messages::{EngineMessage, InterfaceMessage};
use std::sync::mpsc;

pub(crate) fn main_loop(_sender: mpsc::Sender<InterfaceMessage>, _receiver: mpsc::Receiver<EngineMessage>) {
   unimplemented!()
}
