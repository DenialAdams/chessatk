use crate::board::{Move, State};
use std::time::Duration;

// Intraprocess Communication Messages

// Interface to Engine
pub(crate) enum InterfaceMessage {
   GoDepth(u64),   // Calculate until depth and respond with the best move
   GoTime(Duration),
   QueryEval, // Query the evaluation of the current game state
   SetState(State),
}

// Engine to Interface
pub(crate) enum EngineMessage {
   BestMove(Option<Move>),
   CurrentEval(f64),
}
