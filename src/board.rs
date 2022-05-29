use smallvec::SmallVec;
use std::fmt;
use std::str::FromStr;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const PROMOTION_TARGETS: [PromotionTarget; 4] = [
   PromotionTarget::Knight,
   PromotionTarget::Bishop,
   PromotionTarget::Rook,
   PromotionTarget::Queen,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Square {
   Empty,
   BlackPawn,
   WhitePawn,
   BlackKnight,
   WhiteKnight,
   BlackBishop,
   WhiteBishop,
   BlackRook,
   WhiteRook,
   BlackQueen,
   WhiteQueen,
   BlackKing,
   WhiteKing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
   Black,
   White,
}

impl std::ops::Not for Color {
   type Output = Color;
   fn not(self) -> Color {
      match self {
         Color::Black => Color::White,
         Color::White => Color::Black,
      }
   }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Piece {
   Empty,
   Pawn,
   Knight,
   Bishop,
   Rook,
   Queen,
   King,
}

impl Square {
   pub fn piece(self) -> Piece {
      match self {
         Square::Empty => Piece::Empty,
         Square::BlackPawn | Square::WhitePawn => Piece::Pawn,
         Square::BlackKnight | Square::WhiteKnight => Piece::Knight,
         Square::BlackBishop | Square::WhiteBishop => Piece::Bishop,
         Square::BlackRook | Square::WhiteRook => Piece::Rook,
         Square::BlackQueen | Square::WhiteQueen => Piece::Queen,
         Square::BlackKing | Square::WhiteKing => Piece::King,
      }
   }

   pub fn color(self) -> Option<Color> {
      match self {
         Square::Empty => None,
         Square::WhitePawn
         | Square::WhiteKnight
         | Square::WhiteBishop
         | Square::WhiteRook
         | Square::WhiteQueen
         | Square::WhiteKing => Some(Color::White),
         Square::BlackPawn
         | Square::BlackKnight
         | Square::BlackBishop
         | Square::BlackRook
         | Square::BlackQueen
         | Square::BlackKing => Some(Color::Black),
      }
   }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PromotionTarget {
   Knight,
   Bishop,
   Rook,
   Queen,
}

impl fmt::Display for PromotionTarget {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let display = match self {
         PromotionTarget::Knight => "n",
         PromotionTarget::Bishop => "b",
         PromotionTarget::Rook => "r",
         PromotionTarget::Queen => "q",
      };
      write!(f, "{}", display)
   }
}

impl FromStr for PromotionTarget {
   type Err = String;

   fn from_str(s: &str) -> Result<PromotionTarget, String> {
      match s {
         "n" => Ok(PromotionTarget::Knight),
         "b" => Ok(PromotionTarget::Bishop),
         "r" => Ok(PromotionTarget::Rook),
         "q" => Ok(PromotionTarget::Queen),
         _ => Err(format!("Expected one of ASCII nbrq for promotion target, got {}", s)),
      }
   }
}

// TODO: all this crap can drop and we can just derive once const generics
// (actually indexing by u8 is kinda nice)
#[derive(Clone, Copy)]
pub struct Board(pub [Square; 64]);

impl std::ops::Index<u8> for Board {
   type Output = Square;

   fn index(&self, index: u8) -> &Square {
      &self.0[index as usize]
   }
}

impl std::ops::IndexMut<u8> for Board {
   fn index_mut(&mut self, index: u8) -> &mut Square {
      &mut self.0[index as usize]
   }
}

impl PartialEq for Board {
   fn eq(&self, other: &Board) -> bool {
      self.0[..] == other.0[..]
   }
}

impl Eq for Board {}

impl std::hash::Hash for Board {
   fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      self.0[..].hash(state)
   }
}

// ----

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Position {
   pub squares: Board,
   pub white_kingside_castle: bool,
   pub white_queenside_castle: bool,
   pub black_kingside_castle: bool,
   pub black_queenside_castle: bool,
   pub en_passant_square: Option<u8>,
}

impl std::fmt::Display for Position {
   fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      let mut i = 0;
      writeln!(f)?;
      for square in self.squares.0.iter() {
         write!(f, "{:?} ", square)?;
         i += 1;
         if i == 8 {
            i = 0;
            writeln!(f)?;
         }
      }
      Ok(())
   }
}

impl Position {
   #[cfg(test)]
   pub fn from_moves(moves: &str) -> Result<Position, String> {
      let mut position = State::from_start().position;
      for a_str_move in moves.split_whitespace() {
         let a_move: Move = a_str_move.parse()?;
         position.apply_move(a_move);
      }
      Ok(position)
   }

   fn apply_move(&mut self, a_move: Move) {
      // Piece movement
      {
         if let Some(promotion_target) = a_move.promotion {
            // Handle promotion
            let new_piece_white = self.squares[a_move.origin].color() == Some(Color::White);
            match promotion_target {
               PromotionTarget::Knight => {
                  if new_piece_white {
                     self.squares[a_move.destination] = Square::WhiteKnight;
                  } else {
                     self.squares[a_move.destination] = Square::BlackKnight;
                  }
               }
               PromotionTarget::Bishop => {
                  if new_piece_white {
                     self.squares[a_move.destination] = Square::WhiteBishop;
                  } else {
                     self.squares[a_move.destination] = Square::BlackBishop;
                  }
               }
               PromotionTarget::Rook => {
                  if new_piece_white {
                     self.squares[a_move.destination] = Square::WhiteRook;
                  } else {
                     self.squares[a_move.destination] = Square::BlackRook;
                  }
               }
               PromotionTarget::Queen => {
                  if new_piece_white {
                     self.squares[a_move.destination] = Square::WhiteQueen;
                  } else {
                     self.squares[a_move.destination] = Square::BlackQueen;
                  }
               }
            }
         } else {
            // Normal case
            self.squares[a_move.destination] = self.squares[a_move.origin];
         }
         self.squares[a_move.origin] = Square::Empty; // End original piece movement
      }

      // If king moved, do castling rook movement (potentially) and revoke castling rights (always)
      // If pawn moved, do en-passant checking
      self.en_passant_square = None;
      {
         match self.squares[a_move.destination] {
            Square::WhiteKing => {
               self.white_kingside_castle = false;
               self.white_queenside_castle = false;
               if a_move.origin == 60 && a_move.destination == 62 {
                  // WKC
                  self.squares[63] = Square::Empty;
                  self.squares[61] = Square::WhiteRook;
               } else if a_move.origin == 60 && a_move.destination == 58 {
                  // WQC
                  self.squares[56] = Square::Empty;
                  self.squares[59] = Square::WhiteRook;
               }
            }
            Square::BlackKing => {
               self.black_kingside_castle = false;
               self.black_queenside_castle = false;
               if a_move.origin == 4 && a_move.destination == 6 {
                  // BKC
                  self.squares[7] = Square::Empty;
                  self.squares[5] = Square::BlackRook;
               } else if a_move.origin == 4 && a_move.destination == 2 {
                  // BQC
                  self.squares[0] = Square::Empty;
                  self.squares[3] = Square::BlackRook;
               }
            }
            Square::WhitePawn => {
               if a_move.origin - a_move.destination == 16 {
                  self.en_passant_square = Some(a_move.destination + 8);
               } else if Some(a_move.destination) == self.en_passant_square {
                  self.squares[(a_move.destination + 8)] = Square::Empty;
               }
            }
            Square::BlackPawn => {
               if a_move.destination - a_move.origin == 16 {
                  self.en_passant_square = Some(a_move.destination - 8);
               } else if Some(a_move.destination) == self.en_passant_square {
                  self.squares[(a_move.destination - 8)] = Square::Empty;
               }
            }
            _ => {
               // No special treatment needed
            }
         }
      }

      // Revoke castling rights if rook moved or was captured
      {
         if a_move.origin == 63 || a_move.destination == 63 {
            self.white_kingside_castle = false;
         } else if a_move.origin == 56 || a_move.destination == 56 {
            self.white_queenside_castle = false;
         } else if a_move.origin == 7 || a_move.destination == 7 {
            self.black_kingside_castle = false;
         } else if a_move.origin == 0 || a_move.destination == 0 {
            self.black_queenside_castle = false;
         }
      }
   }

   pub fn gen_moves_color(&self, color: Color, do_check_checking: bool) -> Vec<Move> {
      let mut results = Vec::with_capacity(128);
      for (i, square) in self
         .squares
         .0
         .iter()
         .enumerate()
         .filter(|(_, x)| x.color() == Some(color))
      {
         let i = i as u8;
         match square.piece() {
            Piece::Pawn => match color {
               Color::White => {
                  white_pawn_movegen(i, self, &mut results, do_check_checking);
               }
               Color::Black => {
                  black_pawn_movegen(i, self, &mut results, do_check_checking);
               }
            },
            Piece::Knight => {
               let pot_squares = [
                  i + 6,
                  i + 10,
                  i + 15,
                  i + 17,
                  i.wrapping_sub(6),
                  i.wrapping_sub(10),
                  i.wrapping_sub(15),
                  i.wrapping_sub(17),
               ];
               for pot_square in pot_squares
                  .iter()
                  .filter(|x| **x < 64)
                  .filter(|x| !(self.squares[**x].color() == Some(color)))
                  .filter(|x| abs_diff(i % 8, **x % 8) <= 2)
               {
                  let a_move = Move {
                     origin: i,
                     destination: *pot_square,
                     promotion: None,
                  };
                  if do_check_checking {
                     let mut new_board = self.clone();
                     new_board.apply_move(a_move);
                     if !new_board.in_check(color) {
                        results.push(a_move);
                     }
                  } else {
                     results.push(a_move);
                  }
               }
            }
            Piece::Bishop => {
               bishop_movegen(i, self, color, &mut results, do_check_checking);
            }
            Piece::Rook => {
               rook_movegen(i, self, color, &mut results, do_check_checking);
            }
            Piece::Queen => {
               bishop_movegen(i, self, color, &mut results, do_check_checking);
               rook_movegen(i, self, color, &mut results, do_check_checking);
            }
            Piece::King => {
               // Normal movement
               let pot_squares = [
                  i + 1,
                  i + 7,
                  i + 8,
                  i + 9,
                  i.wrapping_sub(1),
                  i.wrapping_sub(7),
                  i.wrapping_sub(8),
                  i.wrapping_sub(9),
               ];
               for pot_square in pot_squares
                  .iter()
                  .filter(|x| **x < 64)
                  .filter(|x| !(self.squares[**x].color() == Some(color)))
                  .filter(|x| abs_diff(i % 8, **x % 8) <= 1)
               {
                  let a_move = Move {
                     origin: i,
                     destination: *pot_square,
                     promotion: None,
                  };
                  if do_check_checking {
                     let mut new_board = self.clone();
                     new_board.apply_move(a_move);
                     if !new_board.in_check(color) {
                        results.push(a_move);
                     }
                  } else {
                     results.push(a_move);
                  }
               }

               // Castling
               // we can never capture a king by castling, so only bother with it when we are generating legal moves
               if do_check_checking {
                  if color == Color::Black {
                     if self.black_kingside_castle
                        && self.squares[5] == Square::Empty
                        && self.squares[6] == Square::Empty
                     {
                        let a_move = Move {
                           origin: 4,
                           destination: 6,
                           promotion: None,
                        };
                        let mut new_board = self.clone();
                        new_board.apply_move(a_move);
                        if !new_board.squares_attacked(color, &[4, 5, 6]) {
                           results.push(a_move);
                        }
                     }
                     if self.black_queenside_castle
                        && self.squares[3] == Square::Empty
                        && self.squares[2] == Square::Empty
                        && self.squares[1] == Square::Empty
                     {
                        let a_move = Move {
                           origin: 4,
                           destination: 2,
                           promotion: None,
                        };
                        let mut new_board = self.clone();
                        new_board.apply_move(a_move);
                        if !new_board.squares_attacked(color, &[4, 3, 2]) {
                           results.push(a_move);
                        }
                     }
                  } else {
                     if self.white_kingside_castle
                        && self.squares[61] == Square::Empty
                        && self.squares[62] == Square::Empty
                     {
                        let a_move = Move {
                           origin: 60,
                           destination: 62,
                           promotion: None,
                        };
                        let mut new_board = self.clone();
                        new_board.apply_move(a_move);
                        if !new_board.squares_attacked(color, &[60, 61, 62]) {
                           results.push(a_move);
                        }
                     }
                     if self.white_queenside_castle
                        && self.squares[59] == Square::Empty
                        && self.squares[58] == Square::Empty
                        && self.squares[57] == Square::Empty
                     {
                        let a_move = Move {
                           origin: 60,
                           destination: 58,
                           promotion: None,
                        };
                        let mut new_board = self.clone();
                        new_board.apply_move(a_move);
                        if !new_board.squares_attacked(color, &[60, 59, 58]) {
                           results.push(a_move);
                        }
                     }
                  }
               }
            }
            Piece::Empty => {
               // No moves
            }
         }
      }
      results
   }

   pub fn squares_attacked(&self, color: Color, squares: &[u8]) -> bool {
      let moves = self.gen_moves_color(!color, false);
      for a_move in moves {
         if squares.contains(&a_move.destination) {
            return true;
         }
      }
      false
   }

   pub fn in_check(&self, color: Color) -> bool {
      let king_pos = self
         .squares
         .0
         .iter()
         .enumerate()
         .find(|(_, x)| x.color() == Some(color) && x.piece() == Piece::King)
         .map(|(i, _)| i as u8)
         .unwrap();
      let moves = self.gen_moves_color(!color, false);
      for a_move in moves {
         if a_move.destination == king_pos {
            return true;
         }
      }
      false
   }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct State {
   pub position: Position,
   pub prior_positions: SmallVec<[Position; 8]>,
   pub side_to_move: Color,
   pub halfmove_clock: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
   origin: u8,
   destination: u8,
   promotion: Option<PromotionTarget>,
}

impl fmt::Display for Move {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      index_to_algebraic(self.origin, f)?;
      index_to_algebraic(self.destination, f)?;
      if let Some(promotion) = self.promotion {
         write!(f, "{}", promotion)?;
      }
      Ok(())
   }
}

impl FromStr for Move {
   type Err = String;

   fn from_str(s: &str) -> Result<Move, String> {
      if s.len() < 4 || s.len() > 5 {
         return Err(format!(
            "A full move has to be 4-5 bytes long, got a move ({}) that was {} bytes long",
            s,
            s.len()
         ));
      }
      let origin_square = &s[..2];
      let dest_square = &s[2..4];
      let promotion_target = s.get(4..5).map(str::parse::<PromotionTarget>);
      let promotion_target = if let Some(result) = promotion_target {
         Some(result?)
      } else {
         None
      };
      Ok(Move {
         origin: algebraic_to_index(origin_square)?,
         destination: algebraic_to_index(dest_square)?,
         promotion: promotion_target,
      })
   }
}

fn index_to_algebraic(index: u8, f: &mut fmt::Formatter) -> fmt::Result {
   write!(
      f,
      "{}",
      match index % 8 {
         0 => "a",
         1 => "b",
         2 => "c",
         3 => "d",
         4 => "e",
         5 => "f",
         6 => "g",
         7 => "h",
         _ => unreachable!(),
      }
   )?;
   write!(f, "{}", (8 - index / 8))
}

fn algebraic_to_index(algebraic: &str) -> Result<u8, String> {
   if algebraic.len() != 2 {
      return Err(format!("{} not a valid algebraic location; too long", algebraic));
   }
   let col: u8 = match algebraic.as_bytes()[0] {
      b'a' => 0,
      b'b' => 1,
      b'c' => 2,
      b'd' => 3,
      b'e' => 4,
      b'f' => 5,
      b'g' => 6,
      b'h' => 7,
      file => return Err(format!("{} is not a valid algebraic file, expected a..=h", file)),
   };
   let row = match algebraic.as_bytes()[1] {
      b'1' => 7,
      b'2' => 6,
      b'3' => 5,
      b'4' => 4,
      b'5' => 3,
      b'6' => 2,
      b'7' => 1,
      b'8' => 0,
      rank => return Err(format!("{} is not a valid algebraic rank, expected 1..=8", rank)),
   };
   Ok((row * 8) + col)
}

impl State {
   #[cfg(test)]
   pub fn from_moves(moves: &str) -> Result<State, String> {
      let mut state = State::from_start();
      for a_str_move in moves.split_whitespace() {
         let a_move: Move = a_str_move.parse()?;
         state = state.apply_move(a_move);
      }
      Ok(state)
   }

   pub fn from_start() -> State {
      State::from_fen(START_FEN).unwrap()
   }

   pub fn apply_moves_from_uci(&self, moves: &str) -> State {
      let mut state = self.clone();
      for a_move in moves.split_whitespace() {
         state = state.apply_move(a_move.parse().unwrap());
      }
      state
   }

   pub fn apply_move(&self, a_move: Move) -> State {
      let (new_halfmove_clock, new_prior_positions) = if self.position.squares[a_move.destination] != Square::Empty
         || self.position.squares[a_move.origin] == Square::BlackPawn
         || self.position.squares[a_move.origin] == Square::WhitePawn
      {
         (0, SmallVec::new())
      } else {
         let mut npp = SmallVec::new();
         npp.push(self.position.clone());
         (self.halfmove_clock + 1, npp)
      };

      let mut new_position = self.position.clone();
      new_position.apply_move(a_move);

      State {
         halfmove_clock: new_halfmove_clock,
         position: new_position,
         prior_positions: new_prior_positions,
         side_to_move: !self.side_to_move,
      }
   }

   pub fn gen_moves(&self, do_check_checking: bool) -> Vec<Move> {
      self.position.gen_moves_color(self.side_to_move, do_check_checking)
   }

   pub fn from_fen(fen: &str) -> Result<State, String> {
      let mut squares = [Square::Empty; 64];
      let mut index = 0;
      // TODO: add string index to error messages
      let fen_sections: Vec<&str> = fen.split_whitespace().collect();
      if fen_sections.len() != 6 {
         return Err(format!(
            "malformed FEN; expected 6 whitespace delimited sections, found {}",
            fen_sections.len()
         ));
      }

      if fen_sections[0].len() > 71 || fen_sections[0].len() < 15 {
         return Err(format!("malformed FEN; length of piece placment section can't be larger than 71 or less than 15 and be a valid board, found length of {}", fen_sections[0].len()));
      }
      for ascii_char in fen_sections[0].bytes() {
         if index == 64 {
            return Err("malformed FEN; too many squares on board".into());
         }
         match ascii_char {
            b'p' => {
               squares[index] = Square::BlackPawn;
               index += 1;
            }
            b'P' => {
               squares[index] = Square::WhitePawn;
               index += 1;
            }
            b'n' => {
               squares[index] = Square::BlackKnight;
               index += 1;
            }
            b'N' => {
               squares[index] = Square::WhiteKnight;
               index += 1;
            }
            b'b' => {
               squares[index] = Square::BlackBishop;
               index += 1;
            }
            b'B' => {
               squares[index] = Square::WhiteBishop;
               index += 1;
            }
            b'r' => {
               squares[index] = Square::BlackRook;
               index += 1;
            }
            b'R' => {
               squares[index] = Square::WhiteRook;
               index += 1;
            }
            b'q' => {
               squares[index] = Square::BlackQueen;
               index += 1;
            }
            b'Q' => {
               squares[index] = Square::WhiteQueen;
               index += 1;
            }
            b'k' => {
               squares[index] = Square::BlackKing;
               index += 1;
            }
            b'K' => {
               squares[index] = Square::WhiteKing;
               index += 1;
            }
            b'1' => {
               index += 1;
            }
            b'2' => {
               index += 2;
            }
            b'3' => {
               index += 3;
            }
            b'4' => {
               index += 4;
            }
            b'5' => {
               index += 5;
            }
            b'6' => {
               index += 6;
            }
            b'7' => {
               index += 7;
            }
            b'8' => {
               index += 8;
            }
            b'/' => {
               if index % 8 != 0 {
                  return Err("malformed FEN; got to end of rank without all squares in rank accounted for".into());
               }
            }
            _ => {
               return Err(format!("malformed FEN; got unexpected byte {} (ASCII: {}) during piece placement, expecting one of ASCII pbnrqkPBNRQK12345678/", ascii_char, ascii_char as char));
            }
         }
      }

      if fen_sections[1].len() != 1 {
         return Err(format!(
            "malformed FEN; expected length of 1 byte for player to move subsection, found length of {}",
            fen_sections[1].len()
         ));
      }

      let to_move = fen_sections[1].as_bytes()[0];
      let side_to_move = match to_move {
         b'w' => Color::White,
         b'b' => Color::Black,
         _ => {
            return Err(format!(
               "malformed FEN; got unexpected byte {} (ASCII: {}) parsing player to move. Expecting one of ASCII wb",
               to_move, to_move as char
            ));
         }
      };

      let castling = fen_sections[2].as_bytes();
      if castling.len() > 4 {
         return Err(format!(
            "malformed FEN; castling rights section shouldn't be longer than 4 bytes or less than 1, found {}",
            castling.len()
         ));
      }

      let mut wkc = false;
      let mut wqc = false;
      let mut bkc = false;
      let mut bqc = false;
      if castling.get(0).cloned() != Some(b'-') {
         for ascii_char in castling.iter() {
            match *ascii_char {
               b'K' => {
                  if wkc {
                     return Err(
                        "malformed FEN; encountered White Kingside castling rights twice when parsing castling rights"
                           .into(),
                     );
                  }
                  wkc = true;
               }
               b'Q' => {
                  if wqc {
                     return Err(
                        "malformed FEN; encountered White Queenside castling rights twice when parsing castling rights"
                           .into(),
                     );
                  }
                  wqc = true;
               }
               b'k' => {
                  if bkc {
                     return Err(
                        "malformed FEN; encountered Black Kingside castling rights twice when parsing castling rights"
                           .into(),
                     );
                  }
                  bkc = true;
               }
               b'q' => {
                  if bqc {
                     return Err(
                        "malformed FEN; encountered Black Queenside castling rights twice when parsing castling rights"
                           .into(),
                     );
                  }
                  bqc = true;
               }
               _ => {
                  return Err(format!(
                  "malformed FEN; found byte {} (ASCII: {}) when parsing castling rights. Expected one of ASCII KQkq",
                  ascii_char, *ascii_char as char
               ));
               }
            }
         }
      }

      let en_passant_square_section = fen_sections[3];
      if en_passant_square_section.len() > 2 {
         return Err(format!(
            "malformed FEN; en passant square shouldn't be longer than 2 bytes or less than 1, found {}",
            en_passant_square_section.len()
         ));
      }
      let en_passant_square = match en_passant_square_section {
         "-" => None,
         algebraic => match algebraic_to_index(algebraic) {
            Ok(index) => Some(index),
            Err(e) => {
               return Err(format!(
                  "malformed FEN; en passant square was not valid algebraic notation: {}",
                  e
               ));
            }
         },
      };

      let halfmove_clock: u64 = match fen_sections[4].parse() {
         Ok(val) => val,
         Err(e) => {
            return Err(format!(
               "malformed FEN; halfmove clock value {} couldn't be parsed as a number: {}",
               fen_sections[4], e
            ));
         }
      };

      Ok(State {
         position: Position {
            squares: Board(squares),
            white_kingside_castle: wkc,
            white_queenside_castle: wqc,
            black_kingside_castle: bkc,
            black_queenside_castle: bqc,
            en_passant_square,
         },
         prior_positions: SmallVec::new(),
         side_to_move,
         halfmove_clock,
      })
   }
}

fn white_pawn_movegen(origin: u8, cur_position: &Position, results: &mut Vec<Move>, do_check_checking: bool) {
   let i = origin;
   if i >= 48 && i <= 55 {
      // 2 SQUARE MOVEMENT
      if cur_position.squares[(i - 16)] == Square::Empty && cur_position.squares[(i - 8)] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i - 16,
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(Color::White) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
      }
   }
   if i >= 8 && i <= 15 {
      // CAPTURE + PROMOTION
      {
         let pot_squares = [i.wrapping_sub(7), i.wrapping_sub(9)];
         for pot_square in pot_squares
            .iter()
            .filter(|x| **x < 64)
            .filter(|x| cur_position.squares[**x].color() == Some(Color::Black))
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            for promotion_target in PROMOTION_TARGETS.iter() {
               let a_move = Move {
                  origin: i,
                  destination: *pot_square,
                  promotion: Some(*promotion_target),
               };
               if do_check_checking {
                  let mut new_board = cur_position.clone();
                  new_board.apply_move(a_move);
                  if !new_board.in_check(Color::White) {
                     results.push(a_move);
                  }
               } else {
                  results.push(a_move);
               }
            }
         }
      }
      // NORMAL MOVEMENT + PROMOTION
      if cur_position.squares[i.wrapping_sub(8)] == Square::Empty {
         for promotion_target in PROMOTION_TARGETS.iter() {
            let a_move = Move {
               origin: i,
               destination: i.wrapping_sub(8),
               promotion: Some(*promotion_target),
            };
            if do_check_checking {
               let mut new_board = cur_position.clone();
               new_board.apply_move(a_move);
               if !new_board.in_check(Color::White) {
                  results.push(a_move);
               }
            } else {
               results.push(a_move);
            }
         }
      }
   } else {
      // CAPTURE (+ EN-PASSANT)
      {
         let pot_squares = [i.wrapping_sub(7), i.wrapping_sub(9)];
         for pot_square in pot_squares
            .iter()
            .filter(|x| **x < 64)
            .filter(|x| {
               cur_position.squares[**x].color() == Some(Color::Black) || cur_position.en_passant_square == Some(**x)
            })
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            let a_move = Move {
               origin: i,
               destination: *pot_square,
               promotion: None,
            };
            if do_check_checking {
               let mut new_board = cur_position.clone();
               new_board.apply_move(a_move);
               if !new_board.in_check(Color::White) {
                  results.push(a_move);
               }
            } else {
               results.push(a_move);
            }
         }
      }
      // NORMAL MOVEMENT
      if cur_position.squares[i.wrapping_sub(8)] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(8),
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(Color::White) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
      }
   }
}

fn black_pawn_movegen(origin: u8, cur_position: &Position, results: &mut Vec<Move>, do_check_checking: bool) {
   let i = origin;
   if i >= 8 && i <= 15 {
      // 2 SQUARE MOVEMENT
      if cur_position.squares[(i + 16)] == Square::Empty && cur_position.squares[(i + 8)] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i + 16,
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(Color::Black) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
      }
   }
   if i >= 48 && i <= 55 {
      // CAPTURE + PROMOTION
      {
         let pot_squares = [i + 7, i + 9];
         for pot_square in pot_squares
            .iter()
            .filter(|x| **x < 64)
            .filter(|x| cur_position.squares[**x].color() == Some(Color::White))
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            for promotion_target in PROMOTION_TARGETS.iter() {
               let a_move = Move {
                  origin: i,
                  destination: *pot_square,
                  promotion: Some(*promotion_target),
               };
               if do_check_checking {
                  let mut new_board = cur_position.clone();
                  new_board.apply_move(a_move);
                  if !new_board.in_check(Color::Black) {
                     results.push(a_move);
                  }
               } else {
                  results.push(a_move);
               }
            }
         }
      }
      // NORMAL MOVEMENT + PROMOTION
      if cur_position.squares[(i + 8)] == Square::Empty {
         for promotion_target in PROMOTION_TARGETS.iter() {
            let a_move = Move {
               origin: i,
               destination: i + 8,
               promotion: Some(*promotion_target),
            };
            if do_check_checking {
               let mut new_board = cur_position.clone();
               new_board.apply_move(a_move);
               if !new_board.in_check(Color::Black) {
                  results.push(a_move);
               }
            } else {
               results.push(a_move);
            }
         }
      }
   } else {
      // CAPTURE (+ EN-PASSANT)
      {
         let pot_squares = [i + 7, i + 9];
         for pot_square in pot_squares
            .iter()
            .filter(|x| **x < 64)
            .filter(|x| {
               cur_position.squares[**x].color() == Some(Color::White) || cur_position.en_passant_square == Some(**x)
            })
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            let a_move = Move {
               origin: i,
               destination: *pot_square,
               promotion: None,
            };
            if do_check_checking {
               let mut new_board = cur_position.clone();
               new_board.apply_move(a_move);
               if !new_board.in_check(Color::Black) {
                  results.push(a_move);
               }
            } else {
               results.push(a_move);
            }
         }
      }
      // NORMAL MOVEMENT
      if cur_position.squares[(i + 8)] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i + 8,
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(Color::Black) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
      }
   }
}

fn bishop_movegen(origin: u8, cur_position: &Position, color: Color, results: &mut Vec<Move>, do_check_checking: bool) {
   let i = origin;
   {
      let mut x = 7;
      let mut last_col = i % 8;
      while i + x < 64 && abs_diff((i + x) % 8, last_col) == 1 {
         if cur_position.squares[(i + x)].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[(i + x)] != Square::Empty {
            break;
         }
         last_col = (i + x) % 8;
         x += 7;
      }
   }
   {
      let mut x = 7;
      let mut last_col = i % 8;
      while i.wrapping_sub(x) < 64 && abs_diff(i.wrapping_sub(x) % 8, last_col) == 1 {
         if cur_position.squares[i.wrapping_sub(x)].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[i.wrapping_sub(x)] != Square::Empty {
            break;
         }
         last_col = i.wrapping_sub(x) % 8;
         x += 7;
      }
   }
   {
      let mut x = 9;
      let mut last_col = i % 8;
      while i + x < 64 && abs_diff((i + x) % 8, last_col) == 1 {
         if cur_position.squares[(i + x)].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[(i + x)] != Square::Empty {
            break;
         }
         last_col = (i + x) % 8;
         x += 9;
      }
   }
   {
      let mut x = 9;
      let mut last_col = i % 8;
      while i.wrapping_sub(x) < 64 && abs_diff(i.wrapping_sub(x) % 8, last_col) == 1 {
         if cur_position.squares[i.wrapping_sub(x)].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[i.wrapping_sub(x)] != Square::Empty {
            break;
         }
         last_col = i.wrapping_sub(x) % 8;
         x += 9;
      }
   }
}

fn rook_movegen(origin: u8, cur_position: &Position, color: Color, results: &mut Vec<Move>, do_check_checking: bool) {
   let i = origin;
   let original_col = i % 8;
   {
      let mut x = 8;
      while i.wrapping_sub(x) < 64 {
         if cur_position.squares[(i.wrapping_sub(x))].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[i.wrapping_sub(x)] != Square::Empty {
            break;
         }
         x += 8
      }
   }
   {
      let mut x = 8;
      while i + x < 64 {
         if cur_position.squares[(i + x)].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[(i + x)] != Square::Empty {
            break;
         }
         x += 8
      }
   }
   {
      let mut x = 1;
      while i + x < 64 && (i + x) % 8 > original_col {
         if cur_position.squares[(i + x)].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[(i + x)] != Square::Empty {
            break;
         }
         x += 1
      }
   }
   {
      let mut x = 1;
      while i.wrapping_sub(x) < 64 && i.wrapping_sub(x) % 8 < original_col {
         if cur_position.squares[(i.wrapping_sub(x))].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         if do_check_checking {
            let mut new_board = cur_position.clone();
            new_board.apply_move(a_move);
            if !new_board.in_check(color) {
               results.push(a_move);
            }
         } else {
            results.push(a_move);
         }
         if cur_position.squares[i.wrapping_sub(x)] != Square::Empty {
            break;
         }
         x += 1
      }
   }
}

fn abs_diff(a: u8, b: u8) -> u8 {
   if a > b {
      a - b
   } else {
      b - a
   }
}

#[cfg(test)]
mod tests {
   use crate::board::*;

   #[test]
   fn algebraic_to_index_conversions() {
      assert_eq!(algebraic_to_index("a8"), Ok(0));
      assert_eq!(algebraic_to_index("e4"), Ok(36));
      assert_eq!(algebraic_to_index("e2"), Ok(52));
      assert_eq!(algebraic_to_index("h1"), Ok(63));
   }

   #[test]
   fn algebraic_to_moves() {
      assert_eq!(
         "e2e4".parse::<Move>(),
         Ok(Move {
            origin: 52,
            destination: 36,
            promotion: None
         })
      );
      assert_eq!(
         "a7a8q".parse::<Move>(),
         Ok(Move {
            origin: 8,
            destination: 0,
            promotion: Some(PromotionTarget::Queen)
         })
      );
      assert_eq!(
         "a7a8n".parse::<Move>(),
         Ok(Move {
            origin: 8,
            destination: 0,
            promotion: Some(PromotionTarget::Knight)
         })
      );
      assert_eq!(
         "a7a8b".parse::<Move>(),
         Ok(Move {
            origin: 8,
            destination: 0,
            promotion: Some(PromotionTarget::Bishop)
         })
      );
      assert_eq!(
         "a7a8r".parse::<Move>(),
         Ok(Move {
            origin: 8,
            destination: 0,
            promotion: Some(PromotionTarget::Rook)
         })
      );
   }

   #[test]
   fn moves_to_algebraic() {
      let letters = ["a", "b", "c", "d", "e", "f", "g", "h"];
      for letter in letters.iter() {
         for i in 1..=8 {
            for letter_2 in letters.iter() {
               for j in 1..=8 {
                  let t_move = format!("{}{}{}{}", letter, i, letter_2, j);
                  assert_eq!(format!("{}", t_move.parse::<Move>().unwrap()), t_move);
               }
            }
         }
      }
   }

   #[test]
   fn parses_valid_fen_ok() {
      use std::fs::File;
      use std::io::{BufRead, BufReader};

      let file = File::open("tests/positions.fen").unwrap();
      let buf_reader = BufReader::new(file);
      for fen in buf_reader.lines() {
         let fen = fen.unwrap();
         let board = State::from_fen(&fen);
         assert!(board.is_ok());
      }
   }

   #[test]
   fn move_gen_test() {
      // TODO To be replaced by a more thorough perft
      let mut a = State::from_start();
      assert_eq!(a.gen_moves(true).len(), 20);
      a.apply_move("e2e4".parse().unwrap());
      assert_eq!(a.gen_moves(true).len(), 20);
      a = State::from_moves("g2g4 e7e5").unwrap();
      assert_eq!(a.gen_moves(true).len(), 21); // -1 because no 2 move pawn, +2 because bishop is free
   }

   #[test]
   fn en_passant_option_is_present() {
      let mut a = Position::from_moves("e2e4").unwrap();
      assert_eq!(a.en_passant_square, Some(44));
      a = Position::from_moves("e2e4 e7e5").unwrap();
      assert_eq!(a.en_passant_square, Some(20));
   }

   #[test]
   fn is_in_check_works() {
      let mut a = Position::from_moves("e2e4").unwrap();
      assert_eq!(a.in_check(Color::White), false);
      assert_eq!(a.in_check(Color::Black), false);
      a = Position::from_moves("e2e4 e4e5 d1h5 a7a6 h5f7").unwrap();
      assert_eq!(a.in_check(Color::White), false);
      assert_eq!(a.in_check(Color::Black), true);
      a = Position::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2").unwrap();
      assert_eq!(a.in_check(Color::White), true);
      assert_eq!(a.in_check(Color::Black), false);
      a = Position::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2 b1d2 f8b4")
         .unwrap();
      assert_eq!(a.in_check(Color::White), false);
      assert_eq!(a.in_check(Color::Black), false);
      a = Position::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2 b1d2 f8b4 d2e4")
         .unwrap();
      assert_eq!(a.in_check(Color::White), true);
      assert_eq!(a.in_check(Color::Black), false);
   }

   #[test]
   fn pawn_seventh_check_bug() {
      let a = Position::from_moves("g2g3 d7d5 g1f3 d5d4 h1g1 b8c6 g1h1 c8g4 f1g2 e7e5 h1f1 e5e4 f3h4 e4e3 h2h3 e3d2")
         .unwrap();
      assert_eq!(a.in_check(Color::White), true);
   }

   #[test]
   fn square_small() {
      assert_eq!(std::mem::size_of::<Square>(), 1)
   }
}
