use crate::board::{Board, Move};
use crate::messages::{EngineMessage, InterfaceMessage};
use std;
use std::sync::mpsc;

pub(crate) fn start(receiver: mpsc::Receiver<InterfaceMessage>, sender: mpsc::Sender<EngineMessage>) {
   let mut board = Board::from_start();
   loop {
      match receiver.recv().unwrap() {
         InterfaceMessage::BoardState(moves) => {
            board = Board::from_moves(&moves).unwrap();
         }
         InterfaceMessage::Go(depth) => {
            let (_eval, best_move) = search(depth, board);
            sender.send(EngineMessage::BestMove(best_move.unwrap())).unwrap();
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
   let moves = board.gen_moves(true);
   if moves.is_empty() {
      max = 0;
   }
   let mut nodes_expanded = 1;
   let mut nodes_generated = moves.len() as u64;
   for (a_move, new_board) in moves {
      let score: i64 = -nega_max(depth - 1, new_board, &mut nodes_expanded, &mut nodes_generated);
      if score > max {
         max = score;
         best_move = Some(a_move);
      }
   }
   println!("nodes generated: {} nodes expanded: {}", nodes_generated, nodes_expanded);
   (max, best_move)
}

fn nega_max(depth: u64, board: Board, nodes_expanded: &mut u64, nodes_generated: &mut u64) -> i64 {
   if depth == 0 {
      return evaluate(board);
   }
   let mut max: i64 = std::i64::MIN;
   let moves = board.gen_moves(true);
   if moves.is_empty() {
      max = 0;
   }
   *nodes_expanded += 1;
   *nodes_generated += moves.len() as u64;
   for (_, new_board) in moves {
      let score: i64 = -nega_max(depth - 1, new_board, nodes_expanded, nodes_generated);
      if score > max {
         max = score;
      }
   }
   max
}

fn evaluate(board: Board) -> i64 {
   use crate::board::Color;

   let num_white_squares = board.squares.iter().filter(|x| x.color() == Some(Color::White)).count();
   let num_black_squares = board.squares.iter().filter(|x| x.color() == Some(Color::Black)).count();
   let eval = num_white_squares as i64 - num_black_squares as i64;
   if board.side_to_move == Color::White {
      eval
   } else {
      -eval
   }
}
