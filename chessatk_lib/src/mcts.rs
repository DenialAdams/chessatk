use crate::board::{Color, Move, State, GameStatus};
use crate::messages::{EngineMessage, InterfaceMessage};
use log::trace;
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
use noisy_float::prelude::*;
use rayon::prelude::*;
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
            let result = mcts(&mut mcts_state, &time_budget, &state, 1.414, &mut thread_rng());
            if state.side_to_move == Color::Black {
               // eval is always relative to side to move, but we want eval to be + for white and - for black
               last_eval = -result.1;
            }
            trace!(
               "finished thinking after {} simulations",
               mcts_state.tree[mcts_state.root].stats.simulations,
            );
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
   last_player: Option<Color>,
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
   tree: Vec<Node>,
   root: usize,
}

impl MctsState {
   fn init() -> MctsState {
      let tree = vec![
         Node {
            last_move: None,
            last_player: None,
            parent: 0,
            children: vec![],
            stats: NodeStats {
               simulations: 0,
               score: 0.0,
            },
         },
      ];

      MctsState {
         tree,
         root: 0,
      }
   }

   fn move_root_down(&mut self, a_move: Move) {
      let new_root = self.tree[self.root]
                  .children
                  .iter()
                  .find(|x| *self.tree[**x].last_move.as_ref().unwrap() == a_move);

      if let Some(n) = new_root {
         self.root = *n;
      } else {
         self.reset();
      }

      trace!("Moved MCTS root. New root has {} simulations", self.tree[self.root].stats.simulations);

      // we could in theory try to "garbage collect" the
      // now dead branches of the tree - not sure if reducing
      // memory usage would help at all, or if it would just
      // be overhead
   }

   fn reset(&mut self) {
      self.tree.clear();

      self.tree.push(Node {
         last_move: None,
         last_player: None,
         parent: 0,
         children: vec![],
         stats: NodeStats {
            simulations: 0,
            score: 0.0,
         },
      });

      self.root = 0;
   }
}


/// panics if there are no legal moves in the given position
fn mcts<R>(mcts_state: &mut MctsState, time_budget: &Duration, state: &State, exploration_val: f64, rng: &mut R) -> (Option<Move>, f64)
where
   R: Rng,
{
   let tree = &mut mcts_state.tree;

   let start = Instant::now();

   while start.elapsed() < *time_budget {
      for _ in 0..100 {
         // determine state
         let mut g = state.clone();

         // select / expand
         let mut cur_node = mcts_state.root;
         let mut moves;
         'outer: loop {
            moves = g.gen_moves(true);

            if moves.is_empty() {
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
                     last_player: Some(g.side_to_move),
                     parent: cur_node,
                     children: vec![],
                     stats: NodeStats {
                        score: 0.0,
                        simulations: 0,
                     },
                  });

                  // since we just created this node, we know it has 0
                  // simulations, and so will always be selected
                  // (so we just cheat and do that selection now)
                  cur_node = new_node_id;
                  g.apply_move(*a_move);
                  break 'outer;
               }
            }

            // go down another layer
            cur_node = *tree[cur_node]
               .children
               .iter()
               .filter(|x| moves.contains(tree[**x].last_move.as_ref().unwrap()))
               .max_by_key(|x| r64(ucb1(exploration_val, tree[**x].stats, tree[cur_node].stats.simulations)))
               .unwrap();
            g = g.apply_move(*tree[cur_node].last_move.as_ref().unwrap());
         }

         // simulate (random rollout)
         let mut g_status = g.status(&moves);
         while g_status == GameStatus::Ongoing {
            let rand_move = moves.choose(rng).unwrap();
            g = g.apply_move(*rand_move);
            moves = g.gen_moves(true);
            g_status = g.status(&moves);
         }

         // backprop
         loop {
            match g_status {
               GameStatus::Draw => {
                  tree[cur_node].stats.score += 0.5;
               }
               GameStatus::Victory(ref p) => {
                  if tree[cur_node].last_player.as_ref() == Some(p) {
                     tree[cur_node].stats.score += 1.0;
                  }
               }
               GameStatus::Ongoing => unreachable!(),
            }
            tree[cur_node].stats.simulations += 1;
            if cur_node == mcts_state.root {
               break;
            }
            cur_node = tree[cur_node].parent;
         }
      }
   }

   let best_child = tree[mcts_state.root]
      .children
      .iter()
      .max_by_key(|x| tree[**x].stats.simulations)
      .unwrap();
   (tree[*best_child].last_move, tree[*best_child].stats.score / tree[*best_child].stats.simulations as f64)
}
