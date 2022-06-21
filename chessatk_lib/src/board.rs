use smallvec::SmallVec;
use std::fmt::{self, Write};
use std::str::FromStr;

const WHITE: usize = 0;
const BLACK: usize = 1;

const PAWN: usize = 0;
const ROOK: usize = 1;
const KNIGHT: usize = 2;
const BISHOP: usize = 3;
const QUEEN: usize = 4;
const KING: usize = 5;

const NORTH: usize = 0;
const SOUTH: usize = 1;
const EAST: usize = 2;
const WEST: usize = 3;
const NORTH_EAST: usize = 4;
const NORTH_WEST: usize = 5;
const SOUTH_EAST: usize = 6;
const SOUTH_WEST: usize = 7;

const RANK_1: u64 = 0xff;
//const RANK_2: u64 = 0xff00;
//const RANK_3: u64 = 0xff0000;
const RANK_4: u64 = 0xff000000;
const RANK_5: u64 = 0xff00000000;
//const RANK_6: u64 = 0xff0000000000;
//const RANK_7: u64 = 0xff000000000000;
const RANK_8: u64 = 0xff00000000000000;

const FILE_H: u64 = 0x8080808080808080;
const FILE_G: u64 = 0x4040404040404040;
//const FILE_F: u64 = 0x2020202020202020;
//const FILE_E: u64 = 0x1010101010101010;
//const FILE_D: u64 = 0x808080808080808;
//const FILE_C: u64 = 0x404040404040404;
const FILE_B: u64 = 0x202020202020202;
const FILE_A: u64 = 0x101010101010101;

const PAWN_ATTACKS: [[u64; 64]; 2] = gen_pawn_attacks();
const KING_ATTACKS: [u64; 64] = gen_king_attacks();
const KNIGHT_ATTACKS: [u64; 64] = gen_knight_attacks();

const RAYS: [[u64; 65]; 8] = gen_rays();

const fn gen_pawn_attacks() -> [[u64; 64]; 2] {
   let mut array: [[u64; 64]; 2] = [[0; 64]; 2];

   // kicking it old school, because const fn...
   let mut i = 0;
   while i < 64 {
      let bb: u64 = 1 << i;

      array[WHITE][i] = ((bb << 9) & !FILE_A) | ((bb << 7) & !FILE_H);
      array[BLACK][i] = ((bb >> 9) & !FILE_H) | ((bb >> 7) & !FILE_A);

      i += 1;
   }
   array
}


const fn gen_king_attacks() -> [u64; 64] {
   let mut array: [u64; 64] = [0; 64];

   let mut i = 0;
   while i < array.len() {
      let bb: u64 = 1 << i;

      array[i] = (((bb << 7) | (bb >> 9) | (bb >> 1)) & (!FILE_H))
         | (((bb << 9) | (bb >> 7) | (bb << 1)) & (!FILE_A))
         | ((bb >> 8) | (bb << 8));

      i += 1;
   }
   array
}

const fn gen_knight_attacks() -> [u64; 64] {
   let mut array: [u64; 64] = [0; 64];

   let mut i = 0;
   while i < array.len() {
      let bb: u64 = 1 << i;

      array[i] = (((bb << 15) | (bb >> 17)) & !FILE_H) | // Left 1
      (((bb >> 15) | (bb << 17)) & !FILE_A) | // Right 1
      (((bb << 6) | (bb >> 10)) & !(FILE_G | FILE_H)) | // Left 2
      (((bb >> 6) | (bb << 10)) & !(FILE_A | FILE_B)); // Right 2

      i += 1;
   }
   array
}

const fn gen_rays() -> [[u64; 65]; 8] {
   let mut array: [[u64; 65]; 8] = [[0; 65]; 8];

   array[NORTH] = gen_north_rays();
   array[SOUTH] = gen_south_rays();
   array[EAST] = gen_east_rays();
   array[WEST] = gen_west_rays();
   array[NORTH_EAST] = gen_north_east_rays();
   array[NORTH_WEST] = gen_north_west_rays();
   array[SOUTH_EAST] = gen_south_east_rays();
   array[SOUTH_WEST] = gen_south_west_rays();

   array
}

const fn gen_north_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut north = 0x0101010101010100;

   let mut i = 0;
   while i < 64 {
      array[i] = north;
      i += 1;
      north <<= 1;
   }

   array
}

const fn gen_south_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut south = 0x0080808080808080;

   let mut i = 64;
   while i > 0 {
      array[i - 1] = south;
      i -= 1;
      south >>= 1;
   }

   array
}

const fn gen_east_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut i = 0;
   while i < 64 {
      array[i] = 2*((1 << (i | 7)) - (1 << i));
      i += 1;
   }

   array
}

const fn gen_west_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut i = 0;
   while i < 64 {
      array[i] = (1 << i) - (1 << (i & 56));
      i += 1;
   }

   array
}

const fn gen_north_east_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut bb = 0x8040201008040200;

   let mut file = 0;
   while file < 8 {
      let mut rank8 = 0;
      let mut bb_inner = bb;
      while rank8 < 64 {
         array[rank8 + file] = bb_inner;
         rank8 += 8;
         bb_inner <<= 8;
      }

      file += 1;
      bb = (bb << 1) & (!FILE_A);
   }

   array
}

const fn gen_south_east_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut bb = 0x2040810204080;

   let mut file = 0;
   while file < 8 {
      let mut rank8 = 56;
      let mut bb_inner = bb;
      loop {
         array[rank8 + file] = bb_inner;
         if rank8 == 0 {
            break;
         }
         rank8 -= 8;
         bb_inner >>= 8;
      }

      file += 1;
      bb = (bb << 1) & (!FILE_A);
   }

   array
}

const fn gen_north_west_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut bb = 0x102040810204000;

   let mut file = 7;
   loop {
      let mut rank8 = 0;
      let mut bb_inner = bb;
      while rank8 < 64 {
         array[rank8 + file] = bb_inner;
         rank8 += 8;
         bb_inner <<= 8;
      }

      if file == 0 {
         break;
      }

      file -= 1;
      bb = (bb >> 1) & (!FILE_H);
   }

   array
}

const fn gen_south_west_rays() -> [u64; 65] {
   let mut array: [u64; 65] = [0; 65];

   let mut bb = 0x40201008040201;

   let mut file = 7;
   loop {
      let mut rank8 = 56;
      let mut bb_inner = bb;
      loop {
         array[rank8 + file] = bb_inner;

         if rank8 == 0 {
            break;
         }
         rank8 -= 8;
         bb_inner >>= 8;
      }

      if file == 0 {
         break;
      }

      file -= 1;
      bb = (bb >> 1) & (!FILE_H);
   }

   array
}

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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
   White,
   Black,
}

impl Color {
   fn as_num(&self) -> usize {
      match self {
        Color::White => WHITE,
        Color::Black => BLACK,
    }
   }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
   Pawn,
   Rook,
   Knight,
   Bishop,
   Queen,
   King,
}

impl Square {
   pub fn piece(self) -> Option<Piece> {
      match self {
         Square::Empty => None,
         Square::BlackPawn | Square::WhitePawn => Some(Piece::Pawn),
         Square::BlackKnight | Square::WhiteKnight => Some(Piece::Knight),
         Square::BlackBishop | Square::WhiteBishop => Some(Piece::Bishop),
         Square::BlackRook | Square::WhiteRook => Some(Piece::Rook),
         Square::BlackQueen | Square::WhiteQueen => Some(Piece::Queen),
         Square::BlackKing | Square::WhiteKing => Some(Piece::King),
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
pub enum PromotionTarget {
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
   pub pieces: [[u64; 6]; 2],
   pub all_pieces: [u64; 2],
   pub attackable: [u64; 2],
   pub occupied: u64,
   pub unoccupied: u64,
}

impl Board {
   /// Not the starting position. All bitboards empty
   fn empty() -> Board {
      Board {
         pieces: [[0; 6]; 2],
         all_pieces: [0; 2],
         attackable: [0; 2],
         occupied: 0,
         unoccupied: 0,
      }
   }

   fn update_derived_bitboards(&mut self) {
      self.all_pieces[WHITE] = self.pieces[WHITE][PAWN]
         | self.pieces[WHITE][ROOK]
         | self.pieces[WHITE][BISHOP]
         | self.pieces[WHITE][QUEEN]
         | self.pieces[WHITE][KING]
         | self.pieces[WHITE][KNIGHT];
      self.all_pieces[BLACK] = self.pieces[BLACK][PAWN]
         | self.pieces[BLACK][ROOK]
         | self.pieces[BLACK][BISHOP]
         | self.pieces[BLACK][QUEEN]
         | self.pieces[BLACK][KING]
         | self.pieces[BLACK][KNIGHT];

      self.attackable[WHITE] = self.all_pieces[WHITE] & !self.pieces[WHITE][KING];
      self.attackable[BLACK] = self.all_pieces[BLACK] & !self.pieces[BLACK][KING];

      self.occupied = self.all_pieces[WHITE] | self.all_pieces[BLACK];
      self.unoccupied = !self.occupied;
   }

   fn remove_piece(&mut self, color: usize, piece: usize, index: u8) {
      let shifted = 1 << index;

      self.pieces[color][piece] ^= shifted;
      self.all_pieces[color] ^= shifted;

      self.occupied ^= shifted;
      self.unoccupied = !self.occupied;

      if piece != KING {
         self.attackable[color] ^= shifted;
      }
   }

   fn add_piece(&mut self, color: usize, piece: usize, index: u8) {
      let shifted = 1 << index;

      self.pieces[color][piece] |= shifted;
      self.all_pieces[color] |= shifted;

      self.occupied |= shifted;
      self.unoccupied = !self.occupied;

      if piece != KING {
         self.attackable[color] ^= shifted;
      }
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
   pub en_passant_square: u64,
   pub side_to_move: Color,
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
      let shifted_origin: u64 = 1 << a_move.origin;
      let shifted_destination: u64 = 1 << a_move.destination;

      let piece_color = ((self.squares.all_pieces[BLACK] & shifted_origin) > 0) as usize;
      let piece_kind = if (self.squares.pieces[piece_color][PAWN] & shifted_origin) > 0 {
         PAWN
      } else if (self.squares.pieces[piece_color][ROOK] & shifted_origin) > 0 {
         ROOK
      } else if (self.squares.pieces[piece_color][KNIGHT] & shifted_origin) > 0 {
         KNIGHT
      } else if (self.squares.pieces[piece_color][BISHOP] & shifted_origin) > 0 {
         BISHOP
      } else if (self.squares.pieces[piece_color][QUEEN] & shifted_origin) > 0 {
         QUEEN
      } else {
         KING
      };

      let destination_piece_color = piece_color ^ 1;
      let destination_piece_kind = if (self.squares.pieces[destination_piece_color][PAWN] & shifted_destination) > 0 {
         Some(PAWN)
      } else if (self.squares.pieces[destination_piece_color][ROOK] & shifted_destination) > 0 {
         Some(ROOK)
      } else if (self.squares.pieces[destination_piece_color][KNIGHT] & shifted_destination) > 0 {
         Some(KNIGHT)
      } else if (self.squares.pieces[destination_piece_color][BISHOP] & shifted_destination) > 0 {
         Some(BISHOP)
      } else if (self.squares.pieces[destination_piece_color][QUEEN] & shifted_destination) > 0 {
         Some(QUEEN)
      } else if (self.squares.pieces[destination_piece_color][KING] & shifted_destination) > 0 {
         Some(KING)
      } else {
         None
      };
      // Piece movement
      {
         self.squares.remove_piece(piece_color, piece_kind, a_move.origin);

         // If this was a capture, need to yeet prior piece
         if let Some(p) = destination_piece_kind {
            self.squares.remove_piece(destination_piece_color, p, a_move.destination);
         }

         if let Some(promotion_target) = a_move.promotion {
            // Handle promotion
            match promotion_target {
               PromotionTarget::Knight => {
                  self.squares.add_piece(piece_color, KNIGHT, a_move.destination);
               }
               PromotionTarget::Bishop => {
                  self.squares.add_piece(piece_color, BISHOP, a_move.destination);
               }
               PromotionTarget::Rook => {
                  self.squares.add_piece(piece_color, ROOK, a_move.destination);
               }
               PromotionTarget::Queen => {
                  self.squares.add_piece(piece_color, QUEEN, a_move.destination);
               }
            }
         } else {
            // Normal case
            self.squares.add_piece(piece_color, piece_kind, a_move.destination);
         }
      }

      println!("-- {}", a_move);
      //println!("{}", bitboard_to_string(self.squares.occupied));

      // If king moved, do castling rook movement (potentially) and revoke castling rights (always)
      // If pawn moved, do en-passant checking
      let old_eps = self.en_passant_square;
      self.en_passant_square = 0;
      {
         match (piece_color, piece_kind) {
            (WHITE, KING) => {
               self.white_kingside_castle = false;
               self.white_queenside_castle = false;
               if a_move.origin == 4 && a_move.destination == 2 {
                  // WQC
                  self.squares.remove_piece(WHITE, ROOK, 0);
                  self.squares.add_piece(WHITE, ROOK, 3);
               } else if a_move.origin == 4 && a_move.destination == 6 {
                  // WKC
                  self.squares.remove_piece(WHITE, ROOK, 7);
                  self.squares.add_piece(WHITE, ROOK, 5);
               }
            }
            (BLACK, KING) => {
               self.black_kingside_castle = false;
               self.black_queenside_castle = false;
               if a_move.origin == 60 && a_move.destination == 62 {
                  // BKC
                  self.squares.remove_piece(BLACK, ROOK, 63);
                  self.squares.add_piece(BLACK, ROOK, 61);
               } else if a_move.origin == 60 && a_move.destination == 58 {
                  // BQC
                  self.squares.remove_piece(BLACK, ROOK, 56);
                  self.squares.add_piece(BLACK, ROOK, 59);
               }
            }
            (WHITE, PAWN) => {
               if a_move.destination - a_move.origin == 16 {
                  self.en_passant_square = 1 << (a_move.origin + 8);
               } else if (1 << a_move.destination) == old_eps {
                  self.squares.remove_piece(BLACK, PAWN, a_move.destination - 8);
               }
            }
            (BLACK, PAWN) => {
               if a_move.origin - a_move.destination == 16 {
                  self.en_passant_square = 1 << (a_move.origin - 8);
               } else if (1 << a_move.destination) == old_eps {
                  self.squares.remove_piece(WHITE, PAWN, a_move.destination + 8);
               }
            }
            _ => {
               // No special treatment needed
            }
         }
      }

      // Revoke castling rights if rook moved or was captured
      {
         if a_move.origin == 7 || a_move.destination == 7 {
            self.white_kingside_castle = false;
         } else if a_move.origin == 0 || a_move.destination == 0 {
            self.white_queenside_castle = false;
         } else if a_move.origin == 63 || a_move.destination == 63 {
            self.black_kingside_castle = false;
         } else if a_move.origin == 56 || a_move.destination == 56 {
            self.black_queenside_castle = false;
         }
      }

      self.side_to_move = !self.side_to_move;
   }

   pub fn gen_moves_color(&self, color: Color, do_check_checking: bool) -> Vec<Move> {
      let mut results = Vec::with_capacity(128);
      match color {
         Color::White => {
            white_pawn_movegen(self, &mut results);
            white_king_movegen(self, &mut results);
            knight_movegen(self, WHITE, &mut results);
            bishop_movegen(self, WHITE, &mut results);
            rook_movegen(self, WHITE, &mut results);
            queen_movegen(self, WHITE, &mut results);
         }
         Color::Black => {
            black_pawn_movegen(self, &mut results);
            black_king_movegen(self, &mut results);
            knight_movegen(self, BLACK, &mut results);
            bishop_movegen(self, BLACK, &mut results);
            rook_movegen(self, BLACK, &mut results);
            queen_movegen(self, BLACK, &mut results);
         }
      }

      if do_check_checking {
         results.drain_filter(|x| {
            let mut cb = self.clone();
            cb.apply_move(*x);
            cb.in_check(color)
         });
      }

      results
   }

   pub fn in_check(&self, color: Color) -> bool {
      let kingdex = self.squares.pieces[color.as_num()][KING].trailing_zeros();
      self.square_is_attacked(color, kingdex as usize)
   }

   pub fn square_is_attacked(&self, defender: Color, square: usize) -> bool {
      let attacker = !defender;

      if PAWN_ATTACKS[defender.as_num()][square] & self.squares.pieces[attacker.as_num()][PAWN] > 0 {
         return true;
      }

      if KNIGHT_ATTACKS[square] & self.squares.pieces[attacker.as_num()][KNIGHT] > 0 {
         return true;
      }

      if KING_ATTACKS[square] & self.squares.pieces[attacker.as_num()][KING] > 0 {
         return true;
      }

      let bishops_and_queens = self.squares.pieces[attacker.as_num()][BISHOP] | self.squares.pieces[attacker.as_num()][QUEEN];
      let rooks_and_queens = self.squares.pieces[attacker.as_num()][ROOK] | self.squares.pieces[attacker.as_num()][QUEEN];

      if (bishop_attacks(self, square) & bishops_and_queens) > 0 {
         return true;
      }

      if (rook_attacks(self, square) & rooks_and_queens) > 0 {
         return true;
      }

      false
   }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct State {
   pub position: Position,
   pub prior_positions: SmallVec<[Position; 8]>,
   pub halfmove_clock: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
   pub origin: u8,
   pub destination: u8,
   pub promotion: Option<PromotionTarget>,
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
   write!(f, "{}", (index / 8) + 1)
}

fn index_to_algebraic_string(index: u8) -> String {
   let mut f = String::new();
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
   ).unwrap();
   write!(f, "{}", (index / 8) + 1).unwrap();
   f
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
      b'1' => 0,
      b'2' => 1,
      b'3' => 2,
      b'4' => 3,
      b'5' => 4,
      b'6' => 5,
      b'7' => 6,
      b'8' => 7,
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

   #[must_use]
   pub fn apply_move(&self, a_move: Move) -> State {
      let is_capture = (self.position.squares.occupied & (1 << a_move.destination)) != 0; 
      let is_pawn_move = ((self.position.squares.pieces[WHITE][PAWN] | self.position.squares.pieces[BLACK][PAWN]) & (1 << a_move.origin)) != 0;
      let (new_halfmove_clock, new_prior_positions) = if is_capture | is_pawn_move
      {
         (0, SmallVec::new())
      } else {
         let mut npp = self.prior_positions.clone();
         npp.push(self.position.clone());
         (self.halfmove_clock + 1, npp)
      };

      let mut new_position = self.position.clone();
      new_position.apply_move(a_move);

      State {
         halfmove_clock: new_halfmove_clock,
         position: new_position,
         prior_positions: new_prior_positions,
      }
   }

   pub fn gen_moves(&self, do_check_checking: bool) -> Vec<Move> {
      self
         .position
         .gen_moves_color(self.position.side_to_move, do_check_checking)
   }

   pub fn from_fen(fen: &str) -> Result<State, String> {
      let mut board = Board::empty();
      let mut index: u64 = 56;
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
         if index > 64 {
            return Err("malformed FEN; too many squares on board".into());
         }
         match ascii_char {
            b'p' => {
               board.pieces[BLACK][PAWN] |= 1 << index;
               index += 1;
            }
            b'P' => {
               board.pieces[WHITE][PAWN] |= 1 << index;
               index += 1;
            }
            b'n' => {
               board.pieces[BLACK][KNIGHT] |= 1 << index;
               index += 1;
            }
            b'N' => {
               board.pieces[WHITE][KNIGHT] |= 1 << index;
               index += 1;
            }
            b'b' => {
               board.pieces[BLACK][BISHOP] |= 1 << index;
               index += 1;
            }
            b'B' => {
               board.pieces[WHITE][BISHOP] |= 1 << index;
               index += 1;
            }
            b'r' => {
               board.pieces[BLACK][ROOK] |= 1 << index;
               index += 1;
            }
            b'R' => {
               board.pieces[WHITE][ROOK] |= 1 << index;
               index += 1;
            }
            b'q' => {
               board.pieces[BLACK][QUEEN] |= 1 << index;
               index += 1;
            }
            b'Q' => {
               board.pieces[WHITE][QUEEN] |= 1 << index;
               index += 1;
            }
            b'k' => {
               board.pieces[BLACK][KING] |= 1 << index;
               index += 1;
            }
            b'K' => {
               board.pieces[WHITE][KING] |= 1 << index;
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
               index -= 16;
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
      if castling.first().cloned() != Some(b'-') {
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
         "-" => 0,
         algebraic => match algebraic_to_index(algebraic) {
            Ok(index) => 1 << index,
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

      board.update_derived_bitboards();

      Ok(State {
         position: Position {
            squares: board,
            white_kingside_castle: wkc,
            white_queenside_castle: wqc,
            black_kingside_castle: bkc,
            black_queenside_castle: bqc,
            en_passant_square,
            side_to_move,
         },
         prior_positions: SmallVec::new(),
         halfmove_clock,
      })
   }

   pub fn to_fen(&self) -> String {
      let mut buf = String::new();
      let mut acc: u8 = 0;
      /*
      for square in self.position.squares.legacy.iter() {
         match square {
            Square::Empty => buf.push('1'),
            Square::BlackPawn => buf.push('p'),
            Square::WhitePawn => buf.push('P'),
            Square::BlackKnight => buf.push('n'),
            Square::WhiteKnight => buf.push('N'),
            Square::BlackBishop => buf.push('b'),
            Square::WhiteBishop => buf.push('B'),
            Square::BlackRook => buf.push('r'),
            Square::WhiteRook => buf.push('R'),
            Square::BlackQueen => buf.push('q'),
            Square::WhiteQueen => buf.push('Q'),
            Square::BlackKing => buf.push('k'),
            Square::WhiteKing => buf.push('K'),
         }
         acc += 1;
         if acc == 8 {
            buf.push('/');
            acc = 0
         }
      } */ // TODO
      buf.push(' ');
      match self.position.side_to_move {
         Color::Black => buf.push('b'),
         Color::White => buf.push('w'),
      }
      buf.push(' ');
      buf.push('-'); // TODO
      buf.push(' ');
      buf.push('-'); // TODO
      buf.push(' ');
      write!(buf, "{}", self.halfmove_clock).unwrap();
      buf.push(' ');
      buf.push('0'); // TODO
      buf
   }

   pub fn status(&self, moves: &[Move]) -> GameStatus {
      if self.prior_positions.iter().filter(|x| **x == self.position).count() >= 2 {
         return GameStatus::Draw;
      }

      if moves.is_empty() && !self.position.in_check(self.position.side_to_move) {
         return GameStatus::Draw;
      }

      if !moves.is_empty() && self.halfmove_clock >= 100 {
         return GameStatus::Draw;
      }

      if moves.is_empty() {
         return GameStatus::Victory(!self.position.side_to_move);
      }

      GameStatus::Ongoing
   }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GameStatus {
   Draw,
   Victory(Color),
   Ongoing,
}

fn pop_lsb(board: &mut u64) -> u32 {
   debug_assert!(*board != 0);
   let lsb_index = board.trailing_zeros();
   *board &= *board - 1;
   lsb_index
}

fn white_pawn_movegen(cur_position: &Position, results: &mut Vec<Move>) {
   // normal movement
   {
      let mut moved_pawns = cur_position.squares.pieces[WHITE][PAWN] << 8;
      moved_pawns &= cur_position.squares.unoccupied;

      let mut promotions = moved_pawns & RANK_8;
      moved_pawns &= !RANK_8;

      while moved_pawns > 0 {
         let to = pop_lsb(&mut moved_pawns);
         results.push(Move {
            origin: (to - 8) as u8,
            destination: to as u8,
            promotion: None,
         });
      }

      while promotions > 0 {
         let to = pop_lsb(&mut promotions);
         results.push(Move {
            origin: (to - 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Queen),
         });
         results.push(Move {
            origin: (to - 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Bishop),
         });
         results.push(Move {
            origin: (to - 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Knight),
         });
         results.push(Move {
            origin: (to - 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Rook),
         });
      }
   }

   // 2 square movement
   {
      let single_pushes = (cur_position.squares.pieces[WHITE][PAWN] << 8) & cur_position.squares.unoccupied;
      let mut double_pushes = (single_pushes << 8) & cur_position.squares.unoccupied & RANK_4;

      while double_pushes > 0 {
         let to = pop_lsb(&mut double_pushes);
         results.push(Move {
            origin: (to - 16) as u8,
            destination: to as u8,
            promotion: None,
         });
      }
   }

   // left attack
   {
      let mut left_regular_attacks =
         (cur_position.squares.pieces[WHITE][PAWN] << 7) & cur_position.squares.attackable[BLACK] & !FILE_H;

      let mut left_attack_promotions = left_regular_attacks & RANK_8;
      left_attack_promotions &= !RANK_8;

      let left_en_passant = (cur_position.squares.pieces[WHITE][PAWN] << 7) & cur_position.en_passant_square & !FILE_H;

      while left_regular_attacks > 0 {
         let to = pop_lsb(&mut left_regular_attacks);
         results.push(Move {
            origin: (to - 7) as u8,
            destination: to as u8,
            promotion: None,
         });
      }

      while left_attack_promotions > 0 {
         let to = pop_lsb(&mut left_attack_promotions);
         results.push(Move {
            origin: (to - 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Queen),
         });
         results.push(Move {
            origin: (to - 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Bishop),
         });
         results.push(Move {
            origin: (to - 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Knight),
         });
         results.push(Move {
            origin: (to - 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Rook),
         });
      }

      if left_en_passant > 0 {
         let to = left_en_passant.trailing_zeros();
         results.push(Move {
            origin: (to - 7) as u8,
            destination: to as u8,
            promotion: None,
         });
      }
   }

   // right attack
   {
      let mut right_regular_attacks =
         (cur_position.squares.pieces[WHITE][PAWN] << 9) & cur_position.squares.attackable[BLACK] & !FILE_A;

      let mut right_attack_promotions = right_regular_attacks & RANK_8;
      right_attack_promotions &= !RANK_8;

      let right_en_passant = (cur_position.squares.pieces[WHITE][PAWN] << 9) & cur_position.en_passant_square & !FILE_A;

      while right_regular_attacks > 0 {
         let to = pop_lsb(&mut right_regular_attacks);
         results.push(Move {
            origin: (to - 9) as u8,
            destination: to as u8,
            promotion: None,
         });
      }

      while right_attack_promotions > 0 {
         let to = pop_lsb(&mut right_attack_promotions);
         results.push(Move {
            origin: (to - 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Queen),
         });
         results.push(Move {
            origin: (to - 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Bishop),
         });
         results.push(Move {
            origin: (to - 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Knight),
         });
         results.push(Move {
            origin: (to - 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Rook),
         });
      }

      if right_en_passant > 0 {
         let to = right_en_passant.trailing_zeros();
         results.push(Move {
            origin: (to - 9) as u8,
            destination: to as u8,
            promotion: None,
         });
      }
   }
}

fn black_pawn_movegen(cur_position: &Position, results: &mut Vec<Move>) {
   // normal movement
   {
      let mut moved_pawns = cur_position.squares.pieces[BLACK][PAWN] >> 8;
      moved_pawns &= cur_position.squares.unoccupied;

      let mut promotions = moved_pawns & RANK_1;
      moved_pawns &= !RANK_1;

      while moved_pawns > 0 {
         let to = pop_lsb(&mut moved_pawns);
         results.push(Move {
            origin: (to + 8) as u8,
            destination: to as u8,
            promotion: None,
         });
      }

      while promotions > 0 {
         let to = pop_lsb(&mut promotions);
         results.push(Move {
            origin: (to + 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Queen),
         });
         results.push(Move {
            origin: (to + 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Bishop),
         });
         results.push(Move {
            origin: (to + 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Knight),
         });
         results.push(Move {
            origin: (to + 8) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Rook),
         });
      }
   }

   // 2 square movement
   {
      let single_pushes = (cur_position.squares.pieces[BLACK][PAWN] >> 8) & cur_position.squares.unoccupied;
      let mut double_pushes = (single_pushes >> 8) & cur_position.squares.unoccupied & RANK_5;

      while double_pushes > 0 {
         let to = pop_lsb(&mut double_pushes);
         results.push(Move {
            origin: (to + 16) as u8,
            destination: to as u8,
            promotion: None,
         });
      }
   }

   // left attack
   {
      let mut left_regular_attacks =
         (cur_position.squares.pieces[BLACK][PAWN] >> 9) & cur_position.squares.attackable[WHITE] & !FILE_H;

      let mut left_attack_promotions = left_regular_attacks & RANK_1;
      left_attack_promotions &= !RANK_1;

      let left_en_passant = (cur_position.squares.pieces[BLACK][PAWN] >> 9) & cur_position.en_passant_square & !FILE_H;

      while left_regular_attacks > 0 {
         let to = pop_lsb(&mut left_regular_attacks);
         results.push(Move {
            origin: (to + 9) as u8,
            destination: to as u8,
            promotion: None,
         });
      }

      while left_attack_promotions > 0 {
         let to = pop_lsb(&mut left_attack_promotions);
         results.push(Move {
            origin: (to + 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Queen),
         });
         results.push(Move {
            origin: (to + 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Bishop),
         });
         results.push(Move {
            origin: (to + 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Knight),
         });
         results.push(Move {
            origin: (to + 9) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Rook),
         });
      }

      if left_en_passant > 0 {
         let to = left_en_passant.trailing_zeros();
         results.push(Move {
            origin: (to + 9) as u8,
            destination: to as u8,
            promotion: None,
         });
      }
   }

   // right attack
   {
      let mut right_regular_attacks =
         (cur_position.squares.pieces[BLACK][PAWN] >> 7) & cur_position.squares.attackable[WHITE] & !FILE_A;

      let mut right_attack_promotions = right_regular_attacks & RANK_8;
      right_attack_promotions &= !RANK_8;

      let right_en_passant = (cur_position.squares.pieces[BLACK][PAWN] >> 7) & cur_position.en_passant_square & !FILE_A;

      while right_regular_attacks > 0 {
         let to = pop_lsb(&mut right_regular_attacks);
         results.push(Move {
            origin: (to + 7) as u8,
            destination: to as u8,
            promotion: None,
         });
      }

      while right_attack_promotions > 0 {
         let to = pop_lsb(&mut right_attack_promotions);
         results.push(Move {
            origin: (to + 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Queen),
         });
         results.push(Move {
            origin: (to + 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Bishop),
         });
         results.push(Move {
            origin: (to + 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Knight),
         });
         results.push(Move {
            origin: (to + 7) as u8,
            destination: to as u8,
            promotion: Some(PromotionTarget::Rook),
         });
      }

      if right_en_passant > 0 {
         let to = right_en_passant.trailing_zeros();
         results.push(Move {
            origin: (to + 7) as u8,
            destination: to as u8,
            promotion: None,
         });
      }
   }
}

fn white_king_movegen(cur_position: &Position, results: &mut Vec<Move>) {
   king_movegen(cur_position, WHITE, results);

   if cur_position.white_kingside_castle {
      let path_bb: u64 = (1 << 5) | (1 << 6);
      let squares_occupied = (path_bb & cur_position.squares.occupied) > 0;
      let squares_attacked = cur_position.square_is_attacked(Color::White, 5) | cur_position.square_is_attacked(Color::White, 6);

      if !squares_occupied & !squares_attacked & !cur_position.in_check(Color::White) {
         results.push(Move {
            origin: 4,
            destination: 6,
            promotion: None,
         });
      }
   }

   if cur_position.white_queenside_castle {
      let path_bb: u64 = (1 << 3) | (1 << 2) | (1 << 1);
      let squares_occupied = (path_bb & cur_position.squares.occupied) > 0;
      let squares_attacked = cur_position.square_is_attacked(Color::White, 3) | cur_position.square_is_attacked(Color::White, 2);

      if !squares_occupied & !squares_attacked & !cur_position.in_check(Color::White) {
         results.push(Move {
            origin: 4,
            destination: 2,
            promotion: None,
         });
      }
   }
}

fn black_king_movegen(cur_position: &Position, results: &mut Vec<Move>) {
   king_movegen(cur_position, BLACK, results);
   
   if cur_position.black_kingside_castle {
      let path_bb: u64 = (1 << 61) | (1 << 62);
      let squares_occupied = (path_bb & cur_position.squares.occupied) > 0;
      let squares_attacked = cur_position.square_is_attacked(Color::Black, 61) | cur_position.square_is_attacked(Color::Black, 62);

      if !squares_occupied & !squares_attacked & !cur_position.in_check(Color::Black) {
         results.push(Move {
            origin: 60,
            destination: 62,
            promotion: None,
         });
      }
   }

   if cur_position.black_queenside_castle {
      let path_bb: u64 = (1 << 57) | (1 << 58) | (1 << 59);
      let squares_occupied = (path_bb & cur_position.squares.occupied) > 0;
      let squares_attacked = cur_position.square_is_attacked(Color::Black, 58) | cur_position.square_is_attacked(Color::Black, 59);

      if !squares_occupied & !squares_attacked & !cur_position.in_check(Color::Black) {
         results.push(Move {
            origin: 60,
            destination: 58,
            promotion: None,
         });
      }
   }
}

fn king_movegen(cur_position: &Position, color: usize, results: &mut Vec<Move>) {
   let king_position = cur_position.squares.pieces[color][KING];
   if king_position == 0 {
      return;
   }

   let king_index = king_position.trailing_zeros();

   let moves = KING_ATTACKS[king_index as usize] & !cur_position.squares.all_pieces[color];

   add_moves(cur_position, color, king_index as u8, moves, results);
}

fn knight_movegen(cur_position: &Position, color: usize, results: &mut Vec<Move>) {
   let mut knights = cur_position.squares.pieces[color][KNIGHT];
   while knights > 0 {
      let origin = pop_lsb(&mut knights);
      let moves = KNIGHT_ATTACKS[origin as usize] & !cur_position.squares.all_pieces[color];
      add_moves(cur_position, color, origin as u8, moves, results);
   }
}

fn positive_ray_attack(direction: usize, square: usize, blockers: u64) -> u64 {
   let mut attacks = RAYS[direction][square];
   let blocked = attacks & blockers;

   let block_square = blocked.trailing_zeros();
   attacks ^= RAYS[direction][block_square as usize];

   attacks
}

fn negative_ray_attack(direction: usize, square: usize, blockers: u64) -> u64 {
   let mut attacks = RAYS[direction][square];
   let blocked = attacks & blockers;

   if blocked > 0 {
      let block_square = blocked.leading_zeros() ^ 63;
      attacks ^= RAYS[direction][block_square as usize];
   }

   attacks
}

fn bitboard_to_string(bb: u64) -> String {
   let mut s = String::new();

   let mut i = 63;
   loop {
      if bb & (1 << i) > 0 {
         s.push('𝟷');
      } else {
         s.push('.');
      }
      if (i) % 8 == 0 {
         s.push('\n');
      }

      if i == 0 {
         break;
      }
      i -= 1;
   }
   s
}

fn bishop_attacks(cur_position: &Position, square: usize) -> u64 {
   positive_ray_attack(NORTH_WEST, square, cur_position.squares.occupied) |
   positive_ray_attack(NORTH_EAST, square, cur_position.squares.occupied) |
   negative_ray_attack(SOUTH_WEST, square, cur_position.squares.occupied) |
   negative_ray_attack(SOUTH_EAST, square, cur_position.squares.occupied)
}

fn rook_attacks(cur_position: &Position, square: usize) -> u64 {
   positive_ray_attack(NORTH, square, cur_position.squares.occupied) |
   positive_ray_attack(EAST, square, cur_position.squares.occupied) |
   negative_ray_attack(SOUTH, square, cur_position.squares.occupied) |
   negative_ray_attack(WEST, square, cur_position.squares.occupied)
}

fn bishop_movegen(cur_position: &Position, color: usize, results: &mut Vec<Move>) {
   let mut bishops = cur_position.squares.pieces[color][BISHOP];
   while bishops > 0 {
      let origin = pop_lsb(&mut bishops);
      let mut moves = bishop_attacks(cur_position, origin as usize);
      moves &= !cur_position.squares.all_pieces[color];
      add_moves(cur_position, color, origin as u8, moves, results);
   }
}

fn rook_movegen(cur_position: &Position, color: usize, results: &mut Vec<Move>) {
   let mut rooks = cur_position.squares.pieces[color][ROOK];
   while rooks > 0 {
      let origin = pop_lsb(&mut rooks);
      let mut moves = rook_attacks(cur_position, origin as usize);
      moves &= !cur_position.squares.all_pieces[color];
      add_moves(cur_position, color, origin as u8, moves, results);
   }
}

fn queen_movegen(cur_position: &Position, color: usize, results: &mut Vec<Move>) {
   let mut queens = cur_position.squares.pieces[color][QUEEN];
   while queens > 0 {
      let origin = pop_lsb(&mut queens);
      let mut moves = bishop_attacks(cur_position, origin as usize) & rook_attacks(cur_position, origin as usize);
      moves &= !cur_position.squares.all_pieces[color];
      add_moves(cur_position, color, origin as u8, moves, results);
   }
}

fn add_moves(
   cur_position: &Position,
   color: usize,
   origin: u8,
   mut moves: u64,
   results: &mut Vec<Move>,
) {
   moves &= !(cur_position.squares.pieces[color ^ 1][KING]);

   while moves > 0 {
      let to = pop_lsb(&mut moves);
      results.push(Move {
         origin,
         destination: to as u8,
         promotion: None,
      });
   }
}

#[cfg(test)]
mod tests {
   use crate::board::*;

   #[test]
   fn algebraic_to_index_conversions() {
      assert_eq!(algebraic_to_index("a8"), Ok(56));
      assert_eq!(algebraic_to_index("e4"), Ok(28));
      assert_eq!(algebraic_to_index("e2"), Ok(12));
      assert_eq!(algebraic_to_index("h1"), Ok(7));
   }

   #[test]
   fn algebraic_to_moves() {
      assert_eq!(
         "e2e4".parse::<Move>(),
         Ok(Move {
            origin: 12,
            destination: 28,
            promotion: None
         })
      );
      assert_eq!(
         "a7a8q".parse::<Move>(),
         Ok(Move {
            origin: 48,
            destination: 56,
            promotion: Some(PromotionTarget::Queen)
         })
      );
      assert_eq!(
         "a7a8n".parse::<Move>(),
         Ok(Move {
            origin: 48,
            destination: 56,
            promotion: Some(PromotionTarget::Knight)
         })
      );
      assert_eq!(
         "a7a8b".parse::<Move>(),
         Ok(Move {
            origin: 48,
            destination: 56,
            promotion: Some(PromotionTarget::Bishop)
         })
      );
      assert_eq!(
         "a7a8r".parse::<Move>(),
         Ok(Move {
            origin: 48,
            destination: 56,
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
   fn movegen_test() {
      let mut a = State::from_start();
      assert_eq!(a.gen_moves(true).len(), 20);
      a = a.apply_move("e2e4".parse().unwrap());
      assert_eq!(a.gen_moves(true).len(), 20);
      a = State::from_moves("g2g4 e7e5").unwrap();
      assert_eq!(a.gen_moves(true).len(), 21); // -1 because no 2 move pawn, +2 because bishop is free
   }

   #[test]
   fn king_movegen_test() {
      let mut a = State::from_fen("8/5k2/8/8/2K5/8/8/8 w - - 0 1").unwrap();
      assert_eq!(a.gen_moves(true).len(), 8);
      a = a.apply_move("c4c5".parse().unwrap());
      assert_eq!(a.gen_moves(true).len(), 8);
   }

   #[test]
   fn en_passant_square() {
      let mut a = Position::from_moves("e2e4").unwrap();
      assert_eq!(a.en_passant_square, 1 << algebraic_to_index("e3").unwrap());
      a = Position::from_moves("e2e4 e7e5").unwrap();
      assert_eq!(a.en_passant_square, 1 << algebraic_to_index("e6").unwrap());
   }

   #[test]
   fn is_in_check_works() {
      let mut a = Position::from_moves("e2e4").unwrap();
      assert!(!a.in_check(Color::White));
      assert!(!a.in_check(Color::Black));
      a = Position::from_moves("e2e4 e4e5 d1h5 a7a6 h5f7").unwrap();
      assert!(!a.in_check(Color::White));
      assert!(a.in_check(Color::Black));
      // rnb1kbnr/p1p2ppp/p7/8/1P2p3/4P3/3q1PPP/RNBQKBNR w KQkq - 0 8
      a = Position::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2").unwrap();
      assert!(a.in_check(Color::White));
      assert!(!a.in_check(Color::Black));
      a = Position::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2 b1d2 f8b4")
         .unwrap();
      assert!(!a.in_check(Color::White));
      assert!(!a.in_check(Color::Black));
      a = Position::from_moves("a2a4 e7e5 a4a5 d7d5 a5a6 b7a6 b2b4 e5e4 c2c3 d5d4 c3d4 d8d4 e2e3 d4d2 b1d2 f8b4 d2e4")
         .unwrap();
      assert!(a.in_check(Color::White));
      assert!(!a.in_check(Color::Black));
   }

   #[test]
   fn pawn_seventh_check_bug() {
      let a = Position::from_moves("g2g3 d7d5 g1f3 d5d4 h1g1 b8c6 g1h1 c8g4 f1g2 e7e5 h1f1 e5e4 f3h4 e4e3 h2h3 e3d2")
         .unwrap();
      assert!(a.in_check(Color::White));
   }

   #[test]
   fn checkmate_no_moves() {
      let game = State::from_fen("2b1kr2/4Qp2/8/pP1Np2p/3P4/3BP3/PP3PPP/R3K2R b KQ - 1 19").unwrap();
      let moves = game.gen_moves(true);
      assert!(moves.is_empty());
   }
}
