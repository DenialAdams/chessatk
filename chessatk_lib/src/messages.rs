use crate::board::{Move, State};
use std::time::Duration;

// Intraprocess Communication Messages

// Interface to Engine
pub enum InterfaceMessage {
   GoDepth(u64), // Calculate until depth and respond with the best move
   GoTime(Duration),
   QueryEval,       // Query the evaluation of the current game state
   ApplyMove(Move), // Incremental state update (for engine optimizations)
   SetState(State), // Full state update
}

// Engine to Interface
pub enum EngineMessage {
   BestMove(Option<Move>),
   CurrentEval(f64),
}
