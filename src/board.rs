use std::fmt;
use std::str::FromStr;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const PROMOTION_TARGETS: [PromotionTarget; 4] = [
   PromotionTarget::Knight,
   PromotionTarget::Bishop,
   PromotionTarget::Rook,
   PromotionTarget::Queen,
];

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy)]
pub struct Board {
   pub squares: [Square; 64],
   pub white_kingside_castle: bool,
   pub white_queenside_castle: bool,
   pub black_kingside_castle: bool,
   pub black_queenside_castle: bool,
   pub en_passant_square: Option<u8>,
   // ----^ need to be hashed for three move repetition checking
   pub side_to_move: Color,
   pub halfmove_clock: u64,
   pub fullmove_number: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
      let promotion_target = s.get(4..5).map(|x| x.parse::<PromotionTarget>());
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

impl Board {
   // TODO: TEMPORARY TO BE DELETED AFTER CONST GENERICS
   pub fn print_board(&self) {
      let mut i = 0;
      println!();
      for square in self.squares.iter() {
         print!("{:?} ", square);
         i += 1;
         if i == 8 {
            i = 0;
            println!();
         }
      }
   }

   pub fn from_start() -> Board {
      Board::from_fen(START_FEN).unwrap()
   }

   pub fn from_moves(moves: &str) -> Result<Board, String> {
      let mut board = Board::from_start();
      for a_str_move in moves.split_whitespace() {
         let a_move: Move = a_str_move.parse()?;
         board = board.apply_move(a_move);
      }
      Ok(board)
   }

   pub fn apply_move(&self, a_move: Move) -> Board {
      // Halfmove clock
      let new_halfmove_clock = if self.squares[a_move.destination as usize] != Square::Empty
         || self.squares[a_move.origin as usize] == Square::BlackPawn
         || self.squares[a_move.origin as usize] == Square::WhitePawn
      {
         0
      } else {
         self.halfmove_clock + 1
      };

      // Fullmove number
      let new_fullmove_number = if self.side_to_move == Color::White {
         self.fullmove_number
      } else {
         self.fullmove_number + 1
      };

      // Piece movement
      let mut new_squares = self.squares;
      {
         if let Some(promotion_target) = a_move.promotion {
            // Handle promotion
            let new_piece_white = self.squares[a_move.origin as usize].color() == Some(Color::White);
            match promotion_target {
               PromotionTarget::Knight => {
                  if new_piece_white {
                     new_squares[a_move.destination as usize] = Square::WhiteKnight
                  } else {
                     new_squares[a_move.destination as usize] = Square::BlackKnight
                  }
               }
               PromotionTarget::Bishop => {
                  if new_piece_white {
                     new_squares[a_move.destination as usize] = Square::WhiteBishop
                  } else {
                     new_squares[a_move.destination as usize] = Square::BlackBishop
                  }
               }
               PromotionTarget::Rook => {
                  if new_piece_white {
                     new_squares[a_move.destination as usize] = Square::WhiteRook
                  } else {
                     new_squares[a_move.destination as usize] = Square::BlackRook
                  }
               }
               PromotionTarget::Queen => {
                  if new_piece_white {
                     new_squares[a_move.destination as usize] = Square::WhiteQueen
                  } else {
                     new_squares[a_move.destination as usize] = Square::BlackQueen
                  }
               }
            }
         } else {
            // Normal case
            new_squares[a_move.destination as usize] = self.squares[a_move.origin as usize];
         }
         new_squares[a_move.origin as usize] = Square::Empty; // End original piece movement
      }

      // Castling
      let mut white_kingside_castle = self.white_kingside_castle;
      let mut white_queenside_castle = self.white_queenside_castle;
      let mut black_kingside_castle = self.black_kingside_castle;
      let mut black_queenside_castle = self.black_queenside_castle;
      // If king moved, do castling rook movement (potentially) and revoke castling rights (always)
      // If pawn moved, do en-passant checking
      let mut en_passant_square = None;
      {
         match new_squares[a_move.destination as usize] {
            Square::WhiteKing => {
               white_kingside_castle = false;
               white_queenside_castle = false;
               if a_move.origin == 60 && a_move.destination == 62 {
                  // WKC
                  new_squares[63] = Square::Empty;
                  new_squares[61] = Square::WhiteRook;
               } else if a_move.origin == 60 && a_move.destination == 58 {
                  // WQC
                  new_squares[56] = Square::Empty;
                  new_squares[59] = Square::WhiteRook;
               }
            }
            Square::BlackKing => {
               black_kingside_castle = false;
               black_queenside_castle = false;
               if a_move.origin == 4 && a_move.destination == 6 {
                  // BKC
                  new_squares[7] = Square::Empty;
                  new_squares[5] = Square::BlackRook;
               } else if a_move.origin == 4 && a_move.destination == 2 {
                  // BQC
                  new_squares[0] = Square::Empty;
                  new_squares[3] = Square::BlackRook;
               }
            }
            Square::WhitePawn => {
               if a_move.origin - a_move.destination == 16 {
                  en_passant_square = Some(a_move.destination + 8);
               } else if Some(a_move.destination) == self.en_passant_square {
                  new_squares[(a_move.destination + 8) as usize] = Square::Empty;
               }
            }
            Square::BlackPawn => {
               if a_move.destination - a_move.origin == 16 {
                  en_passant_square = Some(a_move.destination - 8);
               } else if Some(a_move.destination) == self.en_passant_square {
                  new_squares[(a_move.destination - 8) as usize] = Square::Empty;
               }
            }
            _ => {
               // No special treatment needed
            }
         }
      }

      // Revoke castling rights if rook moved
      {
         if a_move.origin == 63 {
            white_kingside_castle = false;
         } else if a_move.origin == 56 {
            white_queenside_castle = false;
         } else if a_move.origin == 7 {
            black_kingside_castle = false;
         } else if a_move.origin == 0 {
            black_queenside_castle = false;
         }
      }

      Board {
         squares: new_squares,
         white_kingside_castle,
         white_queenside_castle,
         black_kingside_castle,
         black_queenside_castle,
         side_to_move: !self.side_to_move,
         en_passant_square,
         halfmove_clock: new_halfmove_clock,
         fullmove_number: new_fullmove_number,
      }
   }

   pub fn gen_moves(&self, do_check_checking: bool) -> Vec<(Move, Board)> {
      self.gen_moves_color(self.side_to_move, do_check_checking)
   }

   fn gen_moves_color(&self, color: Color, do_check_checking: bool) -> Vec<(Move, Board)> {
      let mut results = Vec::new();
      for (i, square) in self
         .squares
         .iter()
         .enumerate()
         .filter(|(_, x)| x.color() == Some(color))
      {
         let i = i as u8;
         match square.piece() {
            Piece::Pawn => match color {
               Color::White => {
                  white_pawn_movegen(i, &self.squares, &self, &mut results, do_check_checking);
               }
               Color::Black => {
                  black_pawn_movegen(i, &self.squares, &self, &mut results, do_check_checking);
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
                  .filter(|x| !(self.squares[**x as usize].color() == Some(color)))
                  .filter(|x| abs_diff(i % 8, **x % 8) <= 2)
               {
                  let a_move = Move {
                     origin: i,
                     destination: *pot_square,
                     promotion: None,
                  };
                  let new_board = self.apply_move(a_move);
                  if !do_check_checking || !new_board.in_check(color) {
                     results.push((a_move, new_board))
                  }
               }
            }
            Piece::Bishop => {
               bishop_movegen(i, &self.squares, color, &self, &mut results, do_check_checking);
            }
            Piece::Rook => {
               rook_movegen(i, &self.squares, color, &self, &mut results, do_check_checking);
            }
            Piece::Queen => {
               bishop_movegen(i, &self.squares, color, &self, &mut results, do_check_checking);
               rook_movegen(i, &self.squares, color, &self, &mut results, do_check_checking);
            }
            Piece::King => {
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
                  .filter(|x| !(self.squares[**x as usize].color() == Some(color)))
                  .filter(|x| abs_diff(i % 8, **x % 8) <= 1)
               {
                  let a_move = Move {
                     origin: i,
                     destination: *pot_square,
                     promotion: None,
                  };
                  let new_board = self.apply_move(a_move);
                  if !do_check_checking || !new_board.in_check(color) {
                     results.push((a_move, new_board))
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

   pub fn in_check(&self, color: Color) -> bool {
      let king_pos = self
         .squares
         .iter()
         .enumerate()
         .find(|(_, x)| x.color() == Some(color) && x.piece() == Piece::King)
         .unwrap()
         .0 as u8;
      let moves = self.gen_moves_color(!color, false);
      for (a_move, _) in moves {
         if a_move.destination == king_pos {
            return true;
         }
      }
      false
   }

   pub fn from_fen(fen: &str) -> Result<Board, String> {
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
      if !(castling.len() == 1 && castling[0] == b'-') {
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

      let fullmove_number: u64 = match fen_sections[5].parse() {
         Ok(val) => val,
         Err(e) => {
            return Err(format!(
               "malformed FEN; fullmove number value {} couldn't be parsed as a number: {}",
               fen_sections[5], e
            ));
         }
      };

      Ok(Board {
         squares,
         white_kingside_castle: wkc,
         white_queenside_castle: wqc,
         black_kingside_castle: bkc,
         black_queenside_castle: bqc,
         side_to_move,
         en_passant_square,
         halfmove_clock,
         fullmove_number,
      })
   }
}

fn white_pawn_movegen(
   origin: u8,
   squares: &[Square; 64],
   cur_board: &Board,
   results: &mut Vec<(Move, Board)>,
   do_check_checking: bool,
) {
   let i = origin;
   if i >= 48 && i <= 55 {
      // 2 SQUARE MOVEMENT
      if squares[(i - 16) as usize] == Square::Empty && squares[(i - 8) as usize] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i - 16,
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(Color::White) {
            results.push((a_move, new_board))
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
            .filter(|x| cur_board.squares[**x as usize].color() == Some(Color::Black))
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            for promotion_target in PROMOTION_TARGETS.iter() {
               let a_move = Move {
                  origin: i,
                  destination: *pot_square,
                  promotion: Some(*promotion_target),
               };
               let new_board = cur_board.apply_move(a_move);
               if !do_check_checking || !new_board.in_check(Color::White) {
                  results.push((a_move, new_board))
               }
            }
         }
      }
      // NORMAL MOVEMENT + PROMOTION
      if squares[i.wrapping_sub(8) as usize] == Square::Empty {
         for promotion_target in PROMOTION_TARGETS.iter() {
            let a_move = Move {
               origin: i,
               destination: i.wrapping_sub(8),
               promotion: Some(*promotion_target),
            };
            let new_board = cur_board.apply_move(a_move);
            if !do_check_checking || !new_board.in_check(Color::White) {
               results.push((a_move, new_board))
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
               cur_board.squares[**x as usize].color() == Some(Color::Black) || cur_board.en_passant_square == Some(**x)
            })
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            let a_move = Move {
               origin: i,
               destination: *pot_square,
               promotion: None,
            };
            let new_board = cur_board.apply_move(a_move);
            if !do_check_checking || !new_board.in_check(Color::White) {
               results.push((a_move, new_board))
            }
         }
      }
      // NORMAL MOVEMENT
      if squares[i.wrapping_sub(8) as usize] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(8),
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(Color::White) {
            results.push((a_move, new_board))
         }
      }
   }
}

fn black_pawn_movegen(
   origin: u8,
   squares: &[Square; 64],
   cur_board: &Board,
   results: &mut Vec<(Move, Board)>,
   do_check_checking: bool,
) {
   let i = origin;
   if i >= 8 && i <= 15 {
      // 2 SQUARE MOVEMENT
      if squares[(i + 16) as usize] == Square::Empty && squares[(i + 8) as usize] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i + 16,
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(Color::Black) {
            results.push((a_move, new_board))
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
            .filter(|x| squares[**x as usize].color() == Some(Color::White))
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            for promotion_target in PROMOTION_TARGETS.iter() {
               let a_move = Move {
                  origin: i,
                  destination: *pot_square,
                  promotion: Some(*promotion_target),
               };
               let new_board = cur_board.apply_move(a_move);
               if !do_check_checking || !new_board.in_check(Color::Black) {
                  results.push((a_move, new_board))
               }
            }
         }
      }
      // NORMAL MOVEMENT + PROMOTION
      if squares[(i + 8) as usize] == Square::Empty {
         for promotion_target in PROMOTION_TARGETS.iter() {
            let a_move = Move {
               origin: i,
               destination: i + 8,
               promotion: Some(*promotion_target),
            };
            let new_board = cur_board.apply_move(a_move);
            if !do_check_checking || !new_board.in_check(Color::Black) {
               results.push((a_move, new_board))
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
            .filter(|x| squares[**x as usize].color() == Some(Color::White) || cur_board.en_passant_square == Some(**x))
            .filter(|x| abs_diff(i % 8, **x % 8) == 1)
         {
            let a_move = Move {
               origin: i,
               destination: *pot_square,
               promotion: None,
            };
            let new_board = cur_board.apply_move(a_move);
            if !do_check_checking || !new_board.in_check(Color::Black) {
               results.push((a_move, new_board))
            }
         }
      }
      // NORMAL MOVEMENT
      if squares[(i + 8) as usize] == Square::Empty {
         let a_move = Move {
            origin: i,
            destination: i + 8,
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(Color::Black) {
            results.push((a_move, new_board))
         }
      }
   }
}

fn bishop_movegen(
   origin: u8,
   squares: &[Square; 64],
   color: Color,
   cur_board: &Board,
   results: &mut Vec<(Move, Board)>,
   do_check_checking: bool,
) {
   let i = origin;
   {
      let mut x = 7;
      let mut last_col = i % 8;
      while i + x < 64 && abs_diff((i + x) % 8, last_col) == 1 {
         if squares[(i + x) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[(i + x) as usize] != Square::Empty {
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
         if squares[i.wrapping_sub(x) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[i.wrapping_sub(x) as usize] != Square::Empty {
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
         if squares[(i + x) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[(i + x) as usize] != Square::Empty {
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
         if squares[i.wrapping_sub(x) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[i.wrapping_sub(x) as usize] != Square::Empty {
            break;
         }
         last_col = i.wrapping_sub(x) % 8;
         x += 9;
      }
   }
}

fn rook_movegen(
   origin: u8,
   squares: &[Square; 64],
   color: Color,
   cur_board: &Board,
   results: &mut Vec<(Move, Board)>,
   do_check_checking: bool,
) {
   let i = origin;
   let original_col = i % 8;
   {
      let mut x = 8;
      while i.wrapping_sub(x) < 64 {
         if squares[(i.wrapping_sub(x)) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[i.wrapping_sub(x) as usize] != Square::Empty {
            break;
         }
         x += 8
      }
   }
   {
      let mut x = 8;
      while i + x < 64 {
         if squares[(i + x) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[(i + x) as usize] != Square::Empty {
            break;
         }
         x += 8
      }
   }
   {
      let mut x = 1;
      while i + x < 64 && (i + x) % 8 > original_col {
         if squares[(i + x) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i + x,
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[(i + x) as usize] != Square::Empty {
            break;
         }
         x += 1
      }
   }
   {
      let mut x = 1;
      while i.wrapping_sub(x) < 64 && i.wrapping_sub(x) % 8 < original_col {
         if squares[(i.wrapping_sub(x)) as usize].color() == Some(color) {
            break;
         }
         let a_move = Move {
            origin: i,
            destination: i.wrapping_sub(x),
            promotion: None,
         };
         let new_board = cur_board.apply_move(a_move);
         if !do_check_checking || !new_board.in_check(color) {
            results.push((a_move, new_board))
         }
         if squares[i.wrapping_sub(x) as usize] != Square::Empty {
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
         let board = Board::from_fen(&fen);
         assert!(board.is_ok());
      }
   }

   #[test]
   fn move_gen_test() {
      // TODO To be replaced by a more thorough perft
      let mut a = Board::from_start();
      assert_eq!(a.gen_moves(true).len(), 20);
      a = a.apply_move("e2e4".parse().unwrap());
      assert_eq!(a.gen_moves(true).len(), 20);
      a = Board::from_moves("g2g4 e7e5").unwrap();
      assert_eq!(a.gen_moves(true).len(), 21); // -1 because no 2 move pawn, +2 because bishop is free
   }

   #[test]
   fn en_passant_option_is_present() {
      let mut a = Board::from_moves("e2e4").unwrap();
      assert_eq!(a.en_passant_square, Some(44));
      a = Board::from_moves("e2e4 e7e5").unwrap();
      assert_eq!(a.en_passant_square, Some(20));
   }

   #[test]
   fn is_in_check_works() {
      let mut a = Board::from_moves("e2e4").unwrap();
      assert_eq!(a.in_check(Color::White), false);
      assert_eq!(a.in_check(Color::Black), false);
      a = Board::from_moves("e2e4 e4e5 d1h5 a7a6 h5f7").unwrap();
      assert_eq!(a.in_check(Color::White), false);
      assert_eq!(a.in_check(Color::Black), true);
      a = Board::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2").unwrap();
      assert_eq!(a.in_check(Color::White), true);
      assert_eq!(a.in_check(Color::Black), false);
      a = Board::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2 b1d2 f8b4").unwrap();
      assert_eq!(a.in_check(Color::White), false);
      assert_eq!(a.in_check(Color::Black), false);
      a = Board::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2 b1d2 f8b4 d2e4")
         .unwrap();
      assert_eq!(a.in_check(Color::White), true);
      assert_eq!(a.in_check(Color::Black), false);
   }

   #[test]
   fn pawn_seventh_check_bug() {
      let a =
         Board::from_moves("g2g3 d7d5 g1f3 d5d4 h1g1 b8c6 g1h1 c8g4 f1g2 e7e5 h1f1 e5e4 f3h4 e4e3 h2h3 e3d2").unwrap();
      a.print_board();
      assert_eq!(a.in_check(Color::White), true);
   }

   #[test]
   fn square_small() {
      assert_eq!(std::mem::size_of::<Square>(), 1)
   }
}
