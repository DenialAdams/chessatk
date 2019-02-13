use crate::board::{Board, Move};
use crate::messages::{EngineMessage, InterfaceMessage};
use log::trace;
use rayon::prelude::*;
use std;
use std::sync::mpsc;
use std::time::Instant;

pub(crate) fn start(receiver: mpsc::Receiver<InterfaceMessage>, sender: mpsc::Sender<EngineMessage>) {
   let mut original_board = Board::from_start();
   let mut board = original_board;
   loop {
      match receiver.recv().unwrap() {
         InterfaceMessage::NewGameFEN(start_pos) => {
            original_board = Board::from_fen(&start_pos).unwrap();
            board = original_board;
         }
         InterfaceMessage::ApplyMovesFromStart(moves) => {
            board = original_board;
            for a_move in moves.split_whitespace() {
               board = board.apply_move(a_move.parse().unwrap());
            }
         }
         InterfaceMessage::Go(depth) => {
            let (_eval, best_move) = search(depth, board);
            sender.send(EngineMessage::BestMove(best_move)).unwrap();
         }
         InterfaceMessage::QueryEval => {
            // TODO: this seems wrong. we evaluated this at a much greater depth, so we should save that.
            sender.send(EngineMessage::CurrentEval(evaluate(&board))).unwrap();
         }
      }
      //eprintln!("{} -> {} @ {}. {}", best_move.unwrap(), eval, target_depth, board.fullmove_number);
      //board = board.apply_move(best_move.unwrap());
      //target_depth += 1;
   }
}

fn search(depth: u64, board: Board) -> (f64, Option<Move>) {
   let search_time_start = Instant::now();
   let mut max: f64 = std::f64::NEG_INFINITY;
   let mut best_move = None;
   let moves = board.gen_moves(true);
   let mut nodes_expanded = 1;
   let mut nodes_generated = 1 + moves.len() as u64;
   if moves.is_empty() && !board.in_check(board.side_to_move) {
      // stalemate
      max = 0.0;
   }
   let scores: Vec<_> = moves
      .into_par_iter()
      .map(|(a_move, new_board)| {
         let mut ne = 0;
         let mut ng = 0;
         let score: f64 = -nega_max(
            depth - 1,
            new_board,
            std::f64::NEG_INFINITY,
            std::f64::INFINITY,
            &mut ne,
            &mut ng,
         );
         (a_move, score, ne, ng)
      })
      .collect();
   for (a_move, score, ne, ng) in scores {
      nodes_expanded += ne;
      nodes_generated += ng;
      if score >= max {
         max = score;
         best_move = Some(a_move);
      }
   }
   trace!(
      "nodes generated: {} nodes expanded: {}",
      nodes_generated,
      nodes_expanded
   );
   trace!(
      "search @ depth {} took {}",
      depth,
      search_time_start.elapsed().as_float_secs()
   );
   (max, best_move)
}

fn nega_max(
   depth: u64,
   board: Board,
   mut alpha: f64,
   beta: f64,
   nodes_expanded: &mut u64,
   nodes_generated: &mut u64,
) -> f64 {
   if depth == 0 {
      return evaluate(&board);
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

fn evaluate(board: &Board) -> f64 {
   use crate::board::Color;

   // material
   let white_mat_score = board
      .squares
      .iter()
      .filter(|x| x.color() == Some(Color::White))
      .fold(0.0, |acc, x| acc + mat_val(x.piece()));
   let black_mat_score = board
      .squares
      .iter()
      .filter(|x| x.color() == Some(Color::Black))
      .fold(0.0, |acc, x| acc + mat_val(x.piece()));
   let mat_score = white_mat_score as f64 - black_mat_score as f64;

   // distance bonus
   let white_dist_score = board
      .squares
      .iter()
      .enumerate()
      .filter(|(_, x)| x.color() == Some(Color::White))
      .fold(0.0f64, |acc, (i, _)| {
         let row = i / 8;
         let dist = 7 - row;
         acc + dist as f64
      });
   let black_dist_score = board
      .squares
      .iter()
      .enumerate()
      .filter(|(_, x)| x.color() == Some(Color::Black))
      .fold(0.0f64, |acc, (i, _)| {
         let row = i / 8;
         acc + row as f64
      });
   let dist_score = white_dist_score - black_dist_score;

   let final_score = mat_score * 0.9 + dist_score * 0.1;

   if board.side_to_move == Color::White {
      final_score
   } else {
      -final_score
   }
}
