use std;
use std::sync::mpsc;
use board::{Board, Move};
use messages::{EngineMessage, InterfaceMessage};

pub(crate) fn start(receiver: mpsc::Receiver<InterfaceMessage>, sender: mpsc::Sender<EngineMessage>) {
    let mut board = Board::from_start();
    loop {
        match receiver.recv().unwrap() {
            InterfaceMessage::BoardState(moves) => {
                board = Board::from_moves(&moves).unwrap();
            }
            InterfaceMessage::Go(depth) => {
                let (eval, best_move) = search(depth, board.clone());
                sender.send(EngineMessage::BestMove(best_move.unwrap()));
            }
        }
        //eprintln!("{} -> {} @ {}. {}", best_move.unwrap(), eval, target_depth, board.fullmove_number);
        //board = board.apply_move(best_move.unwrap());
        //target_depth += 1;
    }
}

fn search(depth: u64, board: Board) -> (i64, Option<Move>) {
    let mut max: i64 = std::i64::MIN;
    let mut best_move = None;
    let moves = board.gen_moves();
    if moves.is_empty() {
        max = 0;
    }
    for (a_move, new_board) in moves {
        let score: i64 = -nega_max(depth - 1, new_board);
        if score > max {
            max = score;
            best_move = Some(a_move);
        }
    }
    (max, best_move)
}


fn nega_max(depth: u64, board: Board) -> i64 {
    if depth == 0 {
        return evaluate(board)
    }
    let mut max: i64 = std::i64::MIN;
    let moves = board.gen_moves();
    if moves.is_empty() {
        max = 0;
    }
    for (_, new_board) in moves {
        let score: i64 = -nega_max(depth - 1, new_board);
        if score > max {
            max = score;
        }
    }
    max
}

fn evaluate(board: Board) -> i64 {
    let num_white_squares = board.squares.iter().filter(|x| x.is_white_piece()).count();
    let num_black_squares = board.squares.iter().filter(|x| x.is_black_piece()).count();
    let eval = num_white_squares as i64 - num_black_squares as i64;
    if board.white_to_move {
        eval
    } else {
        -1 * eval
    }
}
