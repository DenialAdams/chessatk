use crate::messages::{EngineMessage, InterfaceMessage};
use log::{error, info, trace, warn};
use rand::seq::SliceRandom;
use reqwest::{self, StatusCode};
use serde::Deserialize;
use serde_json;
use std::env;
use std::io::{self, BufRead, BufReader};
use std::sync::mpsc;

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
struct GameFull {
   id: String,
   rated: bool,
   white: Player,
   black: Player,
   state: GameState,
}

#[derive(Deserialize)]
struct GameState {
   moves: String,
}

#[derive(Deserialize)]
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

pub(crate) fn main_loop(sender: mpsc::Sender<InterfaceMessage>, receiver: mpsc::Receiver<EngineMessage>) {
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
   if user.title == Some("BOT".into()) {
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
      }
   }

   loop {
      // Accept first challenge
      let challenge_stream = BufReader::new(
         client
            .get("https://lichess.org/api/stream/event")
            .bearer_auth(&api_token)
            .send()
            .unwrap(),
      );
      for line in challenge_stream.lines() {
         let line = line.unwrap();
         if line.is_empty() {
            continue;
         }
         let event: Event = serde_json::from_str(&line).unwrap();
         match event {
            Event::challenge(challenge_outer) => {
               let challenge_id = challenge_outer.challenge.id;
               if !challenge_outer.challenge.rated {
                  let challenge_accept_res = client
                     .post(&format!("https://lichess.org/api/challenge/{}/accept", challenge_id))
                     .bearer_auth(&api_token)
                     .send()
                     .unwrap();
                  if challenge_accept_res.status() != StatusCode::OK {
                     warn!("Failed to accept challenge. Perhaps the challenge was revoked. Proceeding.")
                  }
               }
            }
            Event::gameStart(game_outer) => {
               let game_id = game_outer.game.id;
               let game_stream = BufReader::new(
                  client
                     .get(&format!("https://lichess.org/api/bot/game/stream/{}", game_id))
                     .bearer_auth(&api_token)
                     .send()
                     .unwrap(),
               );
               let mut we_are_white = false;
               for line in game_stream.lines() {
                  let line = line.unwrap();
                  if line.is_empty() {
                     continue;
                  }
                  let game_event: GameEvent = serde_json::from_str(&line).unwrap();
                  match game_event {
                     GameEvent::gameFull(full_game) => {
                        if full_game.white.id == user_id {
                           we_are_white = true;
                        }
                        sender
                           .send(InterfaceMessage::BoardState(full_game.state.moves.clone()))
                           .unwrap();
                        if (full_game.state.moves.split_whitespace().count() % 2 == 0 && we_are_white)
                           || (full_game.state.moves.split_whitespace().count() % 2 != 0 && !we_are_white)
                        {
                           think_and_move(&client, &game_id, &api_token, &sender, &receiver);
                        }
                     }
                     GameEvent::gameState(game_state) => {
                        sender
                           .send(InterfaceMessage::BoardState(game_state.moves.clone()))
                           .unwrap();
                        if (game_state.moves.split_whitespace().count() % 2 == 0 && we_are_white)
                           || (game_state.moves.split_whitespace().count() % 2 != 0 && !we_are_white)
                        {
                           think_and_move(&client, &game_id, &api_token, &sender, &receiver);
                        }
                     }
                     GameEvent::chatLine(chat_line) => {
                        if chat_line.room == "player" && chat_line.username != username {
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
            }
         }
      }
   }
}

fn think_and_move(
   client: &reqwest::Client,
   game_id: &str,
   api_token: &str,
   sender: &mpsc::Sender<InterfaceMessage>,
   receiver: &mpsc::Receiver<EngineMessage>,
) {
   sender.send(InterfaceMessage::Go(3)).unwrap();
   trace!("Our move! Thinking...");
   let EngineMessage::BestMove(engine_move) = receiver.recv().unwrap();
   trace!("Decided on {}", engine_move);
   let make_move_res = client
      .post(&format!(
         "https://lichess.org/api/bot/game/{}/move/{}",
         game_id, engine_move
      ))
      .bearer_auth(&api_token)
      .send()
      .unwrap();
   if make_move_res.status() != StatusCode::OK {
      error!("tried to make move {} and it was rejected", engine_move);
      panic!();
   }
}
