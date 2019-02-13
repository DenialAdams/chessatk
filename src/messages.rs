use crate::board::Move;

// Intraprocess Communication Messages

// Interface to Engine
pub(crate) enum InterfaceMessage {
   Go(u64),                     // Calculate until depth and respond with the best move
   QueryEval,                   // Query the evaluation of the current game state
   NewGameFEN(String),          // Start a new game (clear seen positions) from FEN
   ApplyMovesFromStart(String), // From the starting board (NewGameFEN), apply these UCI moves
}

// Engine to Interface
pub(crate) enum EngineMessage {
   BestMove(Option<Move>),
   CurrentEval(f64),
}
