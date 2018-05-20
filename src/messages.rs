use board::Move;

// Intraprocess Communication Messages

// Interface to Engine
pub(crate) enum InterfaceMessage {
    BoardState(String), // List of moves from beginning in UCI format
    Go(u64),            // Calculate until depth and respond with the best move
}

// Engine to Interface
pub(crate) enum EngineMessage {
    BestMove(Move),
}
