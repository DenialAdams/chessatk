use crate::board::{Board, Move};
use crate::messages::{EngineMessage, InterfaceMessage};
use std;
use std::sync::mpsc;
use log::trace;

pub(crate) fn start(receiver: mpsc::Receiver<InterfaceMessage>, sender: mpsc::Sender<EngineMessage>) {
   let mut board = Board::from_start();
   loop {
      match receiver.recv().unwrap() {
         InterfaceMessage::BoardState(moves) => {
            board = Board::from_moves(&moves).unwrap();
         }
         InterfaceMessage::Go(depth) => {
            let (_eval, best_move) = search(depth, board);
            sender.send(EngineMessage::BestMove(best_move)).unwrap();
         }
      }
      //eprintln!("{} -> {} @ {}. {}", best_move.unwrap(), eval, target_depth, board.fullmove_number);
      //board = board.apply_move(best_move.unwrap());
      //target_depth += 1;
   }
}

fn search(depth: u64, board: Board) -> (f64, Option<Move>) {
   let mut max: f64 = std::f64::NEG_INFINITY;
   let mut best_move = None;
   let moves = board.gen_moves(true);
   let mut nodes_expanded = 1;
   let mut nodes_generated = 1 + moves.len() as u64;
   if moves.is_empty() && !board.in_check(board.side_to_move) {
      // stalemate
      max = 0.0;
   }
   for (a_move, new_board) in moves {
      let score: f64 = -nega_max(depth - 1, new_board, std::f64::NEG_INFINITY, std::f64::INFINITY, &mut nodes_expanded, &mut nodes_generated);
      if score >= max {
         max = score;
         best_move = Some(a_move);
      }
   }
   trace!("nodes generated: {} nodes expanded: {}", nodes_generated, nodes_expanded);
   (max, best_move)
}

fn nega_max(depth: u64, board: Board, mut alpha: f64, beta: f64, nodes_expanded: &mut u64, nodes_generated: &mut u64) -> f64 {
   if depth == 0 {
      return evaluate(board);
   }
   let mut max: f64 = std::f64::NEG_INFINITY;
   let moves = board.gen_moves(true);
   *nodes_expanded += 1;
   *nodes_generated += moves.len() as u64;
   if moves.is_empty() && !board.in_check(board.side_to_move) {
      // stalemate
      max = 0.0;
   }
   for (_, new_board) in moves {
      let score: f64 = -nega_max(depth - 1, new_board, -beta, -alpha, nodes_expanded, nodes_generated);
      if score > max {
         max = score;
      }
      if max > alpha {
         alpha = max;
      }
      if alpha >= beta {
         break;
      }
   }
   max
}

use crate::board::Piece;

fn mat_val(piece: Piece) -> f64 {
   match piece {
      Piece::Empty => 0.0,
      Piece::Pawn => 1.0,
      Piece::Knight => 3.0,
      Piece::Bishop => 3.0,
      Piece::Rook => 5.0,
      Piece::Queen => 10.0,
      Piece::King => 0.0,
   }
}

fn evaluate(board: Board) -> f64 {
   use crate::board::Color;

   let white_mat_score = board.squares.iter().filter(|x| x.color() == Some(Color::White)).fold(0.0, |acc, x| acc + mat_val(x.piece()));
   let black_mat_score = board.squares.iter().filter(|x| x.color() == Some(Color::Black)).fold(0.0, |acc, x| acc + mat_val(x.piece()));
   let eval = white_mat_score as f64 - black_mat_score as f64;
   if board.side_to_move == Color::White {
      eval
   } else {
      -eval
   }
}
