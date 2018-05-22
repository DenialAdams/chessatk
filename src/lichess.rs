use hyper::header::{Authorization, Bearer, Headers};
use messages::{EngineMessage, InterfaceMessage};
use rand::Rng;
use reqwest::{self, StatusCode};
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::mpsc;

const RESPONSES: [&'static str; 14] = [
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

pub(crate) fn main_loop(
    sender: mpsc::Sender<InterfaceMessage>,
    receiver: mpsc::Receiver<EngineMessage>,
) {
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut out_handle = stdout.lock();
    let mut err_handle = stderr.lock();

    let env_api_token = match env::var("LICHESS_API_TOKEN") {
        Ok(token) => Some(token),
        Err(env::VarError::NotPresent) => {
            // Cool, move on
            None
        }
        Err(env::VarError::NotUnicode(_)) => {
            // TODO WARN
            out_handle.write(b"WARN: Lichess API token environment variable found, but with invalid unicode. Ignoring.\n");
            None
        }
    };

    let api_token = {
        if let Some(token) = env_api_token {
            out_handle
                .write(b"Found lichess api token in environment, using that and proceeding.\n");
            token
        } else {
            let api_token: Result<String, io::Error> = do catch {
                let stdin = io::stdin();
                let mut in_handle = stdin.lock();
                let mut line_buf = String::new();

                out_handle.write(b"Lichess API token: ")?;
                out_handle.flush()?;

                let _ = in_handle.read_line(&mut line_buf)?;
                let _ = line_buf.pop();

                line_buf
            };
            api_token.unwrap()
        }
    };

    let client = reqwest::Client::new();
    let user: User = client
        .get("https://lichess.org/api/account")
        .header(Authorization(Bearer {
            token: api_token.clone(),
        }))
        .send()
        .unwrap()
        .json()
        .unwrap();

    let user_id = user.id;
    let username = user.username;
    if user.title == Some("BOT".into()) {
        out_handle.write(b"Lichess user is a bot account, proceeding\n");
    } else {
        out_handle.write(b"Attempting to upgrade account to bot\n");
        let bot_upgrade_res = client
            .post("https://lichess.org/api/bot/account/upgrade")
            .header(Authorization(Bearer {
                token: api_token.clone(),
            }))
            .send()
            .unwrap();
        if bot_upgrade_res.status() == StatusCode::Ok {
            out_handle.write(b"Upgrade to bot account OK, proceeding\n");
        } else {
            err_handle.write(b"Failed to upgrade account to bot, and account is not already a bot. Can't proceed\n");
        }
    }

    loop {
        // Accept first challenge
        let challenge_stream = BufReader::new(
            client
                .get("https://lichess.org/api/stream/event")
                .header(Authorization(Bearer {
                    token: api_token.clone(),
                }))
                .send()
                .unwrap(),
        );
        for line in challenge_stream.lines() {
            let line = line.unwrap();
            if line.is_empty() {
                continue;
            }
            let event: Event = ::serde_json::from_str(&line).unwrap();
            match event {
                Event::challenge(challenge_outer) => {
                    let challenge_id = challenge_outer.challenge.id;
                    if !challenge_outer.challenge.rated {
                        let challenge_accept_res = client
                            .post(&format!(
                                "https://lichess.org/api/challenge/{}/accept",
                                challenge_id
                            ))
                            .header(Authorization(Bearer {
                                token: api_token.clone(),
                            }))
                            .send()
                            .unwrap();
                        if challenge_accept_res.status() != StatusCode::Ok {
                            out_handle.write(b"Failed to accept challenge. Perhaps the challenge was revoked. Proceeding.\n");
                        }
                    }
                }
                Event::gameStart(game_outer) => {
                    let game_id = game_outer.game.id;
                    let game_stream = BufReader::new(
                        client
                            .get(&format!(
                                "https://lichess.org/api/bot/game/stream/{}",
                                game_id
                            ))
                            .header(Authorization(Bearer {
                                token: api_token.clone(),
                            }))
                            .send()
                            .unwrap(),
                    );
                    let mut we_are_white = None;
                    for line in game_stream.lines() {
                        let line = line.unwrap();
                        if line.is_empty() {
                            continue;
                        }
                        let game_event: GameEvent = ::serde_json::from_str(&line).unwrap();
                        match game_event {
                            GameEvent::gameFull(full_game) => {
                                if full_game.white.id == user_id {
                                    we_are_white = Some(true);
                                } else {
                                    we_are_white = Some(false);
                                }
                                write!(&mut out_handle, "{:?}", full_game.state.moves);
                                out_handle.flush();
                                sender
                                    .send(InterfaceMessage::BoardState(
                                        full_game.state.moves.clone(),
                                    ))
                                    .unwrap();
                                if full_game.state.moves.split_whitespace().count() % 2 == 0
                                    && we_are_white.unwrap()
                                {
                                    sender.send(InterfaceMessage::Go(3)).unwrap();
                                    out_handle.write(b"our move!\n");
                                    out_handle.flush();
                                    let engine_move = match receiver.recv().unwrap() {
                                        EngineMessage::BestMove(a_move) => a_move,
                                    };
                                    let make_move_res = client
                                        .post(&format!(
                                            "https://lichess.org/api/bot/game/{}/move/{}",
                                            game_id, engine_move
                                        ))
                                        .header(Authorization(Bearer {
                                            token: api_token.clone(),
                                        }))
                                        .send()
                                        .unwrap();
                                    if make_move_res.status() != StatusCode::Ok {
                                        write!(
                                            &mut err_handle,
                                            "tried to make move {} and it was rejected\n",
                                            engine_move
                                        );
                                        err_handle.flush();
                                        panic!();
                                    }
                                }
                            }
                            GameEvent::gameState(game_state) => {
                                sender
                                    .send(InterfaceMessage::BoardState(game_state.moves.clone()))
                                    .unwrap();
                                if game_state.moves.split_whitespace().count() % 2 == 0
                                    && we_are_white.unwrap()
                                {
                                    sender.send(InterfaceMessage::Go(3)).unwrap();
                                    out_handle.write(b"our move!\n");
                                    out_handle.flush();
                                    let engine_move = match receiver.recv().unwrap() {
                                        EngineMessage::BestMove(a_move) => a_move,
                                    };
                                    let make_move_res = client
                                        .post(&format!(
                                            "https://lichess.org/api/bot/game/{}/move/{}",
                                            game_id, engine_move
                                        ))
                                        .header(Authorization(Bearer {
                                            token: api_token.clone(),
                                        }))
                                        .send()
                                        .unwrap();
                                    if make_move_res.status() != StatusCode::Ok {
                                        write!(
                                            &mut err_handle,
                                            "tried to make move {} and it was rejected\n",
                                            engine_move
                                        );
                                        err_handle.flush();
                                        panic!();
                                    }
                                }
                            }
                            GameEvent::chatLine(chat_line) => {
                                if chat_line.room == "player" && chat_line.username != username {
                                    let chat_saying =
                                        ::rand::thread_rng().choose(&RESPONSES).unwrap();
                                    let body = [("room", "player"), ("text", chat_saying)];
                                    let challenge_accept_res = client
                                        .post(&format!(
                                            "https://lichess.org/api/bot/game/{}/chat",
                                            game_id
                                        ))
                                        .header(Authorization(Bearer {
                                            token: api_token.clone(),
                                        }))
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
