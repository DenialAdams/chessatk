use crate::board::{Color, GameStatus, Move, State};
use crate::messages::{EngineMessage, InterfaceMessage};
use log::trace;
use noisy_float::prelude::*;
use rand::prelude::SliceRandom;
use std::fs::File;
use std::hint::unreachable_unchecked;
use std::io::{BufWriter, Write};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub fn start(receiver: mpsc::Receiver<InterfaceMessage>, sender: mpsc::Sender<EngineMessage>) {
   let mut state = State::from_start();
   let mut last_eval = 0.0f64;
   let mut mcts_state = MctsState::init();
   while let Ok(message) = receiver.recv() {
      match message {
         InterfaceMessage::GoDepth(_depth) => {
            // doesn't make sense for mcts
            unreachable!()
         }
         InterfaceMessage::GoTime(time_budget) => {
            let result = mcts(&mut mcts_state, &time_budget, &state, 0.3);
            if state.position.side_to_move == Color::Black {
               // eval is always relative to side to move, but we want eval to be + for white and - for black
               last_eval = 1.0 - result.1;
            }
            {
               let tree = mcts_state.tree.lock();
               trace!(
                  "finished thinking after {} simulations. odds of victory: {}%",
                  tree[mcts_state.root].stats.simulations,
                  (1.0 - (tree[mcts_state.root].stats.score / tree[mcts_state.root].stats.simulations as f64)) * 100.0,
               );
            }

            //emit_debug_tree(&mcts_state);

            sender.send(EngineMessage::BestMove(result.0)).unwrap();
         }
         InterfaceMessage::QueryEval => {
            sender.send(EngineMessage::CurrentEval(last_eval)).unwrap();
         }
         InterfaceMessage::SetState(new_state) => {
            mcts_state.reset();
            state = new_state;
         }
         InterfaceMessage::ApplyMove(m) => {
            mcts_state.move_root_down(m);
            state = state.apply_move(m);
         }
      }
   }
}

struct Node {
   last_move: Option<Move>,
   last_player: Color,
   parent: usize,
   children: Vec<usize>,
   stats: NodeStats,
}

#[derive(Clone, Copy)]
struct NodeStats {
   simulations: u64,
   score: f64,
}

fn ucb1(exploration_val: f64, node_stats: NodeStats, parent_simulations: u64) -> f64 {
   let win_rate = node_stats.score / node_stats.simulations as f64;
   let exploration_score = exploration_val * ((parent_simulations as f64).ln() / node_stats.simulations as f64).sqrt();
   win_rate + exploration_score
}

struct MctsState {
   tree: parking_lot::Mutex<Vec<Node>>,
   root: usize,
}

impl MctsState {
   fn init() -> MctsState {
      MctsState {
         tree: parking_lot::Mutex::new(Vec::new()),
         root: 0,
      }
   }

   fn move_root_down(&mut self, a_move: Move) {
      let mut tree = self.tree.lock();

      let new_root = tree.get(self.root).and_then(|x| {
         x.children
            .iter()
            .find(|y| *tree[**y].last_move.as_ref().unwrap() == a_move)
      });

      if let Some(n) = new_root {
         self.root = *n;
      } else {
         tree.clear();
         self.root = 0;
      }

      trace!(
         "Moved MCTS root. New root has {} simulations",
         tree.get(self.root).map(|x| x.stats.simulations).unwrap_or(0)
      );

      // we could in theory try to "garbage collect" the
      // now dead branches of the tree - not sure if reducing
      // memory usage would help at all, or if it would just
      // be overhead
   }

   fn reset(&mut self) {
      let mut tree = self.tree.lock();

      tree.clear();
      self.root = 0;
   }
}

/// panics if there are no legal moves in the given position
fn mcts(
   mcts_state: &mut MctsState,
   time_budget: &Duration,
   state: &State,
   exploration_val: f64,
) -> (Option<Move>, f64) {
   {
      let mut tree = mcts_state.tree.lock();
      if tree.len() == 0 {
         tree.push(Node {
            last_move: None,
            last_player: !state.position.side_to_move,
            parent: 0,
            children: vec![],
            stats: NodeStats {
               simulations: 0,
               score: 0.0,
            },
         });
      }
   }
   std::thread::scope(|s| {
      for _ in 0..16 {
         s.spawn(|| {
            mcts_inner(mcts_state, time_budget, state, exploration_val);
         });
      }
   });
   let tree = mcts_state.tree.lock();
   let best_child = tree[mcts_state.root]
      .children
      .iter()
      .max_by_key(|x| tree[**x].stats.simulations)
      .unwrap();
   (
      tree[*best_child].last_move,
      tree[*best_child].stats.score / tree[*best_child].stats.simulations as f64,
   )
}

fn mcts_inner(mcts_state: &MctsState, time_budget: &Duration, state: &State, exploration_val: f64) {
   let mut rng = rand::thread_rng();

   let start = Instant::now();

   while start.elapsed() < *time_budget {
      for _ in 0..100 {
         // determine state
         let mut g = state.clone();

         // select / expand
         let mut cur_node = mcts_state.root;
         let mut moves;
         let mut g_status;

         {
            let mut tree = mcts_state.tree.lock();
            'outer: loop {
               tree[cur_node].stats.simulations += 1; // simulations are updated early, so other threads are aware
               // ("WATCH THE UNOBSERVED: A SIMPLE APPROACH TO PARALLELIZING MONTE CARLO TREE SEARCH")
               moves = g.gen_moves(true);
               g_status = g.status(&moves);

               if g_status != GameStatus::Ongoing {
                  // terminal node
                  break;
               }

               for a_move in moves.iter() {
                  if !tree[cur_node]
                     .children
                     .iter()
                     .any(|x| tree[*x].last_move.as_ref().unwrap() == a_move)
                  {
                     let new_node_id = tree.len();
                     tree[cur_node].children.push(new_node_id);
                     tree.push(Node {
                        last_move: Some(*a_move),
                        last_player: g.position.side_to_move,
                        parent: cur_node,
                        children: vec![],
                        stats: NodeStats {
                           score: 0.0,
                           simulations: 1,
                        },
                     });

                     // select the newly created node
                     cur_node = new_node_id;
                     g = g.apply_move(*a_move);
                     moves = g.gen_moves(true);
                     g_status = g.status(&moves);
                     break 'outer;
                  }
               }

               // go down another layer
               cur_node = *tree[cur_node]
                  .children
                  .iter()
                  .max_by_key(|x| r64(ucb1(exploration_val, tree[**x].stats, tree[cur_node].stats.simulations)))
                  .unwrap();
               g = g.apply_move(*tree[cur_node].last_move.as_ref().unwrap());
            }
         }

         // simulate (random rollout)
         while g_status == GameStatus::Ongoing {
            let rand_move = *moves.choose(&mut rng).unwrap();
            g = g.apply_move(rand_move);
            moves = g.gen_moves(true);
            g_status = g.status(&moves);
         }

         let mut tree = mcts_state.tree.lock();

         // backprop
         loop {
            match g_status {
               GameStatus::Draw => {
                  tree[cur_node].stats.score += 0.5;
               }
               GameStatus::Victory(ref p) => {
                  if tree[cur_node].last_player == *p {
                     tree[cur_node].stats.score += 1.0;
                  }
               }
               GameStatus::Ongoing => unsafe { unreachable_unchecked() },
            }
            if cur_node == mcts_state.root {
               break;
            }
            cur_node = tree[cur_node].parent;
         }
      }
   }
}

fn emit_debug_tree(mcts_state: &MctsState) {
   let out_f = std::fs::File::create("mcts.html").unwrap();
   let mut out = BufWriter::new(out_f);
   writeln!(
      out,
      "<!DOCTYPE HTML>
<html lang=\"en\">
<head>
   <meta charset=\"utf-8\">
   <title>MCTS debug</title>
   <link rel=\"stylesheet\" href=\"./ast.css\">
</head>
<body>"
   )
   .unwrap();
   writeln!(out, "<ul class=\"tree\">").unwrap();

   let tree = mcts_state.tree.lock();
   emit_debug_node(&mut out, mcts_state.root, &tree);

   writeln!(out, "</body>\n</html>").unwrap();
}

fn emit_debug_node(out: &mut BufWriter<File>, i: usize, tree: &[Node]) {
   let node = &tree[i];
   if node.last_move.is_some() {
      writeln!(
         out,
         "<li><span>{}</span><br><span>score «{}» simulations «{}»</span>",
         node.last_move.unwrap(),
         node.stats.score,
         node.stats.simulations
      )
      .unwrap();
   } else {
      writeln!(
         out,
         "<li><span>score «{}» simulations «{}»</span>",
         node.stats.score, node.stats.simulations
      )
      .unwrap();
   }
   writeln!(out, "<ul>").unwrap();
   for child in node.children.iter().copied() {
      emit_debug_node(out, child, tree);
   }
   writeln!(out, "</ul></li>").unwrap();
}
