use crate::board::{Color, Move, State};
use crate::messages::{EngineMessage, InterfaceMessage};
use log::trace;
use rayon::prelude::*;
use std;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub(crate) fn start(receiver: mpsc::Receiver<InterfaceMessage>, sender: mpsc::Sender<EngineMessage>) {
   let mut state = State::from_start();
   let mut last_eval = 0.0f64;
   while let Ok(message) = receiver.recv() {
      match message {
         InterfaceMessage::GoDepth(depth) => {
            let (eval, best_move) = search(depth, &state);
            if state.side_to_move == Color::Black {
               // eval is always relative to side to move, but we want eval to be + for white and - for black
               last_eval = -eval;
            }
            sender.send(EngineMessage::BestMove(best_move)).unwrap();
         }
         InterfaceMessage::GoTime(time_budget) => {
            let mut used_time = Duration::from_secs(0);
            let mut depth = 1;
            let (mut overall_eval, mut overall_best_move) = (0.0, None);
            while used_time * 2 < time_budget {
               let start = Instant::now();
               let (eval, best_move) = search(depth, &state);
               overall_eval = eval;
               overall_best_move = best_move;
               depth += 1;
               used_time += start.elapsed();
            }
            if state.side_to_move == Color::Black {
               // eval is always relative to side to move, but we want eval to be + for white and - for black
               last_eval = -overall_eval;
            }
            sender.send(EngineMessage::BestMove(overall_best_move)).unwrap();
         }
         InterfaceMessage::QueryEval => {
            sender.send(EngineMessage::CurrentEval(last_eval)).unwrap();
         }
         InterfaceMessage::SetState(new_state) => {
            state = new_state;
         }
      }
      //eprintln!("{} -> {} @ {}. {}", best_move.unwrap(), eval, target_depth, board.fullmove_number);
      //board = board.apply_move(best_move.unwrap());
      //target_depth += 1;
   }
}

fn search(depth: u64, state: &State) -> (f64, Option<Move>) {
   if state.prior_positions.iter().filter(|x| **x == state.position).count() >= 2 {
      return (0.0, None);
   }
   let search_time_start = Instant::now();
   let mut max: f64 = std::f64::NEG_INFINITY;
   let mut best_move = None;
   let moves = state.gen_moves(true);
   let mut nodes_expanded = 1;
   let mut nodes_generated = 1 + moves.len() as u64;
   if moves.is_empty() && !state.in_check(state.side_to_move) {
      return (0.0, None);
   }
   if !moves.is_empty() && state.halfmove_clock >= 100 {
      return (0.0, None);
   }
   let scores: Vec<_> = moves
      .into_par_iter()
      .map(|(a_move, new_board)| {
         let mut ne = 0;
         let mut ng = 0;
         let score = -nega_max(
            depth - 1,
            1,
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
   dist_from_root: u64,
   state: State,
   mut alpha: f64,
   beta: f64,
   nodes_expanded: &mut u64,
   nodes_generated: &mut u64,
) -> f64 {
   if state.prior_positions.iter().filter(|x| **x == state.position).count() >= 2 {
      return 0.0;
   }
   if depth == 0 {
      return evaluate(&state);
   }
   let mut max: f64 = -10000.0 + dist_from_root as f64;
   let mut moves = state.gen_moves(true);
   moves.sort_unstable_by(|x, y| evaluate(&x.1).partial_cmp(&evaluate(&y.1)).unwrap());
   *nodes_expanded += 1;
   *nodes_generated += moves.len() as u64;
   if moves.is_empty() && !state.in_check(state.side_to_move) {
      // stalemate
      return 0.0;
   }
   if !moves.is_empty() && state.halfmove_clock >= 100 {
      return 0.0;
   }
   for (_, new_board) in moves {
      let score = -nega_max(depth - 1, dist_from_root + 1, new_board, -beta, -alpha, nodes_expanded, nodes_generated);
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

fn evaluate(state: &State) -> f64 {
   use crate::board::Color;

   let mut white_mat_score = 0.0;
   let mut black_mat_score = 0.0;
   let mut white_dist_score = 0.0;
   let mut black_dist_score = 0.0;
   for (i, square) in state.position.squares.0.iter().enumerate() {
      if square.color() == Some(Color::White) {
         white_mat_score += mat_val(square.piece());

         let row = i / 8;
         let dist = 7 - row;
         white_dist_score += dist as f64;
      } else if square.color() == Some(Color::Black) {
         black_mat_score += mat_val(square.piece());

         let row = i / 8;
         black_dist_score += row as f64;
      }
   }

   let mat_score = white_mat_score as f64 - black_mat_score as f64;
   let dist_score = white_dist_score - black_dist_score;
   let final_score = mat_score * 0.9 + dist_score * 0.1;

   if state.side_to_move == Color::White {
      final_score
   } else {
      -final_score
   }
}
