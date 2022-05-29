use crate::board::{Color, State};
use crate::messages::{EngineMessage, InterfaceMessage};
use fnv::FnvHashSet;
use log::{error, info, trace, warn};
use rand::seq::SliceRandom;
use reqwest::{self, StatusCode};
use serde::Deserialize;
use serde_json;
use std::env;
use std::io::{self, BufRead, BufReader};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const RESPONSES: [&str; 14] = [
   "if you think i'm moving righteous then",
   "i think i'm moving backwards and",
   "i feel you dancing in my chest",
   "it's kind of hurting",
   "and by the way I drag my head",
   "you think i would be grounded and",
   "i would consider severing",
   "to lose the friction",
   "i don't require you to step out side of yourself",
   "my lazy death in front of me had all the time to sigh with ease",
   "and oh i know it'll break my knees",
   "achieved inside of one night's work",
   "i stumbled and began this lurk",
   "it's something of an aimless sort",
];

#[derive(Debug, Deserialize)]
struct User {
   id: String,
   username: String,
   title: Option<String>,
}

#[derive(Deserialize)]
struct ChallengeOuter {
   challenge: ChallengeInner,
}

#[derive(Deserialize)]
struct ChallengeInner {
   id: String,
   rated: bool,
   variant: Variant,
}

#[derive(Deserialize)]
struct Variant {
   key: String,
}

#[derive(Deserialize)]
struct Game {
   id: String,
}

#[derive(Deserialize)]
struct GameStart {
   game: Game,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[allow(non_camel_case_types)]
enum Event {
   challenge(ChallengeOuter),
   gameStart(GameStart),
}

#[derive(Deserialize)]
struct Player {
   id: String,
   name: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct GameFull {
   id: String,
   rated: bool,
   white: Player,
   black: Player,
   initialFen: String,
   state: GameState,
}

#[derive(Deserialize)]
struct GameState {
   moves: String,
   wtime: u64,
   btime: u64,
}

#[derive(Debug, Deserialize)]
struct ChatLine {
   username: String,
   text: String,
   room: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[allow(non_camel_case_types)]
enum GameEvent {
   gameFull(GameFull),
   gameState(GameState),
   chatLine(ChatLine),
}

type EngineInterface = Arc<Mutex<(mpsc::Sender<InterfaceMessage>, mpsc::Receiver<EngineMessage>)>>;

#[allow(clippy::cognitive_complexity)] //temporary, we need to replace with async
pub(crate) fn main_loop(sender: mpsc::Sender<InterfaceMessage>, receiver: mpsc::Receiver<EngineMessage>) {
   let engine_interface: EngineInterface = Arc::new(Mutex::new((sender, receiver)));

   let env_api_token = match env::var("LICHESS_API_TOKEN") {
      Ok(token) => Some(token),
      Err(env::VarError::NotPresent) => {
         // Cool, move on
         None
      }
      Err(env::VarError::NotUnicode(_)) => {
         // TODO WARN
         warn!("Lichess API token environment variable found, but with invalid unicode. Ignoring.");
         None
      }
   };

   let api_token = {
      if let Some(token) = env_api_token {
         info!("Found lichess api token in environment, using that and proceeding.");
         token
      } else {
         let api_token: Result<String, io::Error> = try {
            let mut line_buf = String::new();

            println!("Lichess API token: ");

            let _ = io::stdin().read_line(&mut line_buf)?;
            let _ = line_buf.pop();

            line_buf
         };
         api_token.unwrap()
      }
   };

   let client = reqwest::Client::new();
   let user: User = client
      .get("https://lichess.org/api/account")
      .bearer_auth(&api_token)
      .send()
      .unwrap()
      .json()
      .unwrap();

   let user_id = user.id;
   let username = user.username;
   if user.title.as_ref().map(String::as_str) == Some("BOT") {
      info!("Lichess user is a bot account, proceeding.");
   } else {
      info!("Attempting to upgrade account to bot...");
      let bot_upgrade_res = client
         .post("https://lichess.org/api/bot/account/upgrade")
         .bearer_auth(&api_token)
         .send()
         .unwrap();
      if bot_upgrade_res.status() == StatusCode::OK {
         info!("Upgrade to bot account OK, proceeding.");
      } else {
         error!("Failed to upgrade account to bot, and account is not already a bot. Can't proceed.");
         return;
      }
   }

   let games_in_progress = Arc::new(Mutex::new(FnvHashSet::with_hasher(Default::default())));
   // Accept first challenge
   // TODO: we are silently ignoring errors by being flat
   loop {
      let challenge_stream = BufReader::new(
         client
            .get("https://lichess.org/api/stream/event")
            .bearer_auth(&api_token)
            .send()
            .unwrap(),
      );
      for event in challenge_stream
         .lines()
         .flatten()
         .filter(|x| !x.is_empty())
         .flat_map(|x| serde_json::from_str(&x))
      {
         match event {
            Event::challenge(challenge_outer) => {
               let challenge_id = challenge_outer.challenge.id;
               if challenge_outer.challenge.rated
                  || (challenge_outer.challenge.variant.key != "standard"
                     && challenge_outer.challenge.variant.key != "fromPosition")
               {
                  let challenge_reject_res = client
                     .post(&format!("https://lichess.org/api/challenge/{}/decline", challenge_id))
                     .bearer_auth(&api_token)
                     .send()
                     .unwrap();
                  if challenge_reject_res.status() != StatusCode::OK {
                     warn!("Failed to reject challenge. Perhaps the challenge was revoked. Proceeding.")
                  }
               }
               if games_in_progress.lock().unwrap().len() >= 2 {
                  continue;
               }
               let challenge_accept_res = client
                  .post(&format!("https://lichess.org/api/challenge/{}/accept", challenge_id))
                  .bearer_auth(&api_token)
                  .send()
                  .unwrap();
               if challenge_accept_res.status() != StatusCode::OK {
                  warn!("Failed to accept challenge. Perhaps the challenge was revoked. Proceeding.")
               }
            }
            Event::gameStart(game_outer) => {
               {
                  if games_in_progress.lock().unwrap().contains(&game_outer.game.id) {
                     continue;
                  }
               }
               let cc = client.clone();
               let atc = api_token.clone();
               let uc = username.clone();
               let uidc = user_id.clone();
               let eic = engine_interface.clone();
               {
                  games_in_progress.lock().unwrap().insert(game_outer.game.id.clone());
               }
               let gipc = games_in_progress.clone();
               trace!("joining game {}", game_outer.game.id);
               std::thread::spawn(move || {
                  manage_game(cc, game_outer.game.id, atc, uc, uidc, eic, gipc);
               });
            }
         }
      }
      std::thread::sleep(Duration::from_secs(1));
   }
}

fn manage_game(
   client: reqwest::Client,
   game_id: String,
   api_token: String,
   username: String,
   user_id: String,
   ei: EngineInterface,
   games_in_progress: Arc<Mutex<FnvHashSet<String>>>,
) {
   let game_stream = BufReader::new(
      client
         .get(&format!("https://lichess.org/api/bot/game/stream/{}", game_id))
         .bearer_auth(&api_token)
         .send()
         .unwrap(),
   );
   let mut us_color = Color::Black;
   let mut initial_game_state = State::from_start();
   for line in game_stream.lines() {
      let line = line.unwrap();
      let line = line.trim();
      if line.is_empty() {
         continue;
      }
      let game_event = serde_json::from_str(&line).unwrap();
      match game_event {
         GameEvent::gameFull(full_game) => {
            if full_game.white.id == user_id {
               us_color = Color::White;
            }
            initial_game_state = if full_game.initialFen == "startpos" {
               State::from_start()
            } else {
               State::from_fen(&full_game.initialFen).unwrap()
            };
            let remaining_time = Duration::from_millis(match us_color {
               Color::White => full_game.state.wtime,
               Color::Black => full_game.state.btime,
            });
            let cur_game_state = initial_game_state.apply_moves_from_uci(&full_game.state.moves);
            if cur_game_state.side_to_move == us_color {
               think_and_move(&client, &game_id, &api_token, &ei, cur_game_state, remaining_time);
            }
         }
         GameEvent::gameState(game_state_json) => {
            let remaining_time = Duration::from_millis(match us_color {
               Color::White => game_state_json.wtime,
               Color::Black => game_state_json.btime,
            });
            let cur_game_state = initial_game_state.apply_moves_from_uci(&game_state_json.moves);
            if cur_game_state.side_to_move == us_color {
               think_and_move(&client, &game_id, &api_token, &ei, cur_game_state, remaining_time);
            }
         }
         GameEvent::chatLine(chat_line) => {
            if chat_line.text == "!eval" {
               let ei = ei.lock().unwrap();
               ei.0.send(InterfaceMessage::QueryEval).unwrap();
               let eval = match ei.1.recv().unwrap() {
                  EngineMessage::CurrentEval(e) => e,
                  _ => panic!("expected current eval from the engine!"),
               };
               let body = [("room", &chat_line.room), ("text", &eval.to_string())];
               let _chat_res = client
                  .post(&format!("https://lichess.org/api/bot/game/{}/chat", game_id))
                  .bearer_auth(&api_token)
                  .form(&body)
                  .send()
                  .unwrap();
            } else if chat_line.room == "player" && chat_line.username != username && chat_line.username != "lichess" {
               let chat_saying = RESPONSES.choose(&mut rand::thread_rng()).unwrap();
               let body = [("room", "player"), ("text", chat_saying)];
               let _chat_res = client
                  .post(&format!("https://lichess.org/api/bot/game/{}/chat", game_id))
                  .bearer_auth(&api_token)
                  .form(&body)
                  .send()
                  .unwrap();
            }
         }
      }
   }
   games_in_progress.lock().unwrap().remove(&game_id);
}

fn think_and_move(
   client: &reqwest::Client,
   game_id: &str,
   api_token: &str,
   ei: &EngineInterface,
   state: State,
   remaining_time: Duration,
) {
   let ei = ei.lock().unwrap();
   ei.0.send(InterfaceMessage::SetState(state)).unwrap();
   ei.0.send(InterfaceMessage::GoTime(remaining_time / 20)).unwrap();
   trace!("Our move! Thinking...");
   let e_move = match ei.1.recv().unwrap() {
      EngineMessage::BestMove(best_move_opt) => {
         if let Some(best_move) = best_move_opt {
            best_move
         } else {
            // probably end of game
            // could be bug in the engine
            return;
         }
      }
      _ => panic!("expected a move in response from the engine!"),
   };
   trace!("Decided on {}", e_move);
   let make_move_res = client
      .post(&format!("https://lichess.org/api/bot/game/{}/move/{}", game_id, e_move))
      .bearer_auth(&api_token)
      .send()
      .unwrap();
   if make_move_res.status() != StatusCode::OK {
      error!(
         "tried to make move {} and it was rejected. resigning and moving on",
         e_move
      );
      let _resign_res = client
         .post(&format!("https://lichess.org/api/bot/game/{}/resign", game_id))
         .bearer_auth(&api_token)
         .send()
         .unwrap();
   }
}
