use crate::board::{Color, CompressedMove, Move, Position, State, WHITE, PAWN, KNIGHT, BISHOP, ROOK, QUEEN, BLACK, RANK_8, RANK_5, RANK_6, RANK_7, RANK_4, RANK_3, RANK_2, KING, RANK_1};
use crate::messages::{EngineMessage, InterfaceMessage};
use log::trace;
use rayon::prelude::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub fn start(receiver: mpsc::Receiver<InterfaceMessage>, sender: mpsc::Sender<EngineMessage>) {
   let mut state = State::from_start();
   let mut last_eval = 0.0f64;
   while let Ok(message) = receiver.recv() {
      match message {
         InterfaceMessage::GoDepth(depth) => {
            let (eval, best_move) = search(depth, &state);
            if state.position.side_to_move == Color::Black {
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
            if state.position.side_to_move == Color::Black {
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
         InterfaceMessage::ApplyMove(m) => {
            state.apply_move(m);
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
   let mut moves: Vec<CompressedMove> = Vec::new();
   state.gen_moves(&mut moves);
   let mut nodes_expanded = 1;
   let mut nodes_generated = 1 + moves.len() as u64;
   if moves.is_empty() && !state.position.in_check(state.position.side_to_move) {
      return (0.0, None);
   }
   if !moves.is_empty() && state.halfmove_clock >= 100 {
      return (0.0, None);
   }
   let scores: Vec<_> = moves
      .into_par_iter()
      .map(|a_move| {
         let mut new_state = state.clone();
         new_state.apply_move(a_move.extract());
         let mut ne = 0;
         let mut ng = 0;
         let score = -nega_max(
            depth - 1,
            1,
            new_state,
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
   if let Some(b) = best_move {
      trace!(
         "search @ depth {} took {}. best move: {}",
         depth,
         search_time_start.elapsed().as_secs_f64(),
         b.extract()
      );
   } else {
      trace!(
         "search @ depth {} took {}. game over",
         depth,
         search_time_start.elapsed().as_secs_f64(),
      );
   }
   (max, best_move.map(|x| x.extract()))
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
      return evaluate(&state.position, state.position.side_to_move);
   }
   let mut max: f64 = -10000.0 + dist_from_root as f64;
   let mut moves: Vec<CompressedMove> = Vec::new();
   state.gen_moves(&mut moves);
   //moves.sort_unstable_by(|x, y| evaluate(&x.1).partial_cmp(&evaluate(&y.1)).unwrap());
   *nodes_expanded += 1;
   *nodes_generated += moves.len() as u64;
   if moves.is_empty() && !state.position.in_check(state.position.side_to_move) {
      // stalemate
      return 0.0;
   }
   if !moves.is_empty() && state.halfmove_clock >= 100 {
      return 0.0;
   }
   for a_move in moves {
      let mut state = state.clone();
      state.apply_move(a_move.extract());

      let score = -nega_max(
         depth - 1,
         dist_from_root + 1,
         state,
         -beta,
         -alpha,
         nodes_expanded,
         nodes_generated,
      );
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
      Piece::Pawn => 1.0,
      Piece::Knight => 3.0,
      Piece::Bishop => 3.0,
      Piece::Rook => 5.0,
      Piece::Queen => 10.0,
      Piece::King => 0.0,
   }
}

fn evaluate(position: &Position, side_to_move: Color) -> f64 {
   let mut white_mat_score = 0.0;
   let mut black_mat_score = 0.0;
   let mut white_dist_score = 0.0;
   let mut black_dist_score = 0.0;

   {
      white_mat_score += position.squares.pieces[WHITE][PAWN].count_ones() as f64 * mat_val(Piece::Pawn);
      white_mat_score += position.squares.pieces[WHITE][KNIGHT].count_ones() as f64 * mat_val(Piece::Knight);
      white_mat_score += position.squares.pieces[WHITE][BISHOP].count_ones() as f64 * mat_val(Piece::Bishop);
      white_mat_score += position.squares.pieces[WHITE][ROOK].count_ones() as f64 * mat_val(Piece::Rook);
      white_mat_score += position.squares.pieces[WHITE][QUEEN].count_ones() as f64 * mat_val(Piece::Queen);
   }

   {
      black_mat_score += position.squares.pieces[BLACK][PAWN].count_ones() as f64 * mat_val(Piece::Pawn);
      black_mat_score += position.squares.pieces[BLACK][KNIGHT].count_ones() as f64 * mat_val(Piece::Knight);
      black_mat_score += position.squares.pieces[BLACK][BISHOP].count_ones() as f64 * mat_val(Piece::Bishop);
      black_mat_score += position.squares.pieces[BLACK][ROOK].count_ones() as f64 * mat_val(Piece::Rook);
      black_mat_score += position.squares.pieces[BLACK][QUEEN].count_ones() as f64 * mat_val(Piece::Queen);
   }

   {
      white_dist_score += (position.squares.all_pieces[WHITE] & !position.squares.pieces[WHITE][KING] & RANK_8).count_ones() as f64 * 7.0;
      white_dist_score += (position.squares.all_pieces[WHITE] & !position.squares.pieces[WHITE][KING] & RANK_7).count_ones() as f64 * 6.0;
      white_dist_score += (position.squares.all_pieces[WHITE] & !position.squares.pieces[WHITE][KING] & RANK_6).count_ones() as f64 * 5.0;
      white_dist_score += (position.squares.all_pieces[WHITE] & !position.squares.pieces[WHITE][KING] & RANK_5).count_ones() as f64 * 4.0;
      white_dist_score += (position.squares.all_pieces[WHITE] & !position.squares.pieces[WHITE][KING] & RANK_4).count_ones() as f64 * 3.0;
      white_dist_score += (position.squares.all_pieces[WHITE] & !position.squares.pieces[WHITE][KING] & RANK_3).count_ones() as f64 * 2.0;
      white_dist_score += (position.squares.all_pieces[WHITE] & !position.squares.pieces[WHITE][KING] & RANK_2).count_ones() as f64;
   }

   {
      black_dist_score += (position.squares.all_pieces[BLACK] & !position.squares.pieces[BLACK][KING] & RANK_1).count_ones() as f64 * 7.0;
      black_dist_score += (position.squares.all_pieces[BLACK] & !position.squares.pieces[BLACK][KING] & RANK_2).count_ones() as f64 * 6.0;
      black_dist_score += (position.squares.all_pieces[BLACK] & !position.squares.pieces[BLACK][KING] & RANK_3).count_ones() as f64 * 5.0;
      black_dist_score += (position.squares.all_pieces[BLACK] & !position.squares.pieces[BLACK][KING] & RANK_4).count_ones() as f64 * 4.0;
      black_dist_score += (position.squares.all_pieces[BLACK] & !position.squares.pieces[BLACK][KING] & RANK_5).count_ones() as f64 * 3.0;
      black_dist_score += (position.squares.all_pieces[BLACK] & !position.squares.pieces[BLACK][KING] & RANK_6).count_ones() as f64 * 2.0;
      black_dist_score += (position.squares.all_pieces[BLACK] & !position.squares.pieces[BLACK][KING] & RANK_7).count_ones() as f64;
   }

   let mat_score = white_mat_score as f64 - black_mat_score as f64;
   let dist_score = white_dist_score - black_dist_score;


   let mut move_buf = Vec::new();
   position.gen_moves_color(Color::White, &mut move_buf);
   let white_mobility_score = move_buf.len();
   position.gen_moves_color(Color::Black, &mut move_buf);
   let black_mobility_score = move_buf.len();
   let mobility_score: f64 = white_mobility_score as f64 - black_mobility_score as f64;

   let final_score = dist_score * 0.04 + mobility_score * 0.06 + mat_score * 0.9;

   if side_to_move == Color::White {
      final_score
   } else {
      -final_score
   }
}
