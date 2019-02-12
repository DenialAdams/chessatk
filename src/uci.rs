use crate::messages::{EngineMessage, InterfaceMessage};
use std::io::{self, BufRead, Write};
use std::sync::mpsc;

pub(crate) fn main_loop(_sender: mpsc::Sender<InterfaceMessage>, _receiver: mpsc::Receiver<EngineMessage>) {
   let stdin = io::stdin();
   let mut in_handle = stdin.lock();
   let mut line_buf = String::new();

   let stdout = io::stdout();
   let mut out_handle = stdout.lock();

   loop {
      match in_handle.read_line(&mut line_buf) {
         Ok(bytes_read) => {
            if bytes_read == 0 {
               break;
            }
            let res: Result<(), io::Error> = try {
               let args: Vec<&str> = line_buf.split_whitespace().collect();
               if args.len() == 0 {
                  continue;
               }
               match args[0] {
                  "uci" => {
                     out_handle.write(b"id name chessatk\n")?;
                     out_handle.write(b"id author Richard McCormack\n")?;
                     out_handle.write(b"uciok\n")?;
                  }
                  "isready" => {
                     out_handle.write(b"readyok\n")?;
                  }
                  "position" => {
                     match args[1] {
                        "fen" => {}
                        "startpos" => {}
                        _ => {
                           // TODO ERROR
                           eprintln!("Expected 'fen' or 'startpos' following 'postion'");
                           break;
                        }
                     }
                     /*
                     if let Some(value) = args.get(2) {
                         if *value == "moves" {

                         } else {
                             eprintln!("Expected 'moves' following a position")
                         }
                     } */
                  }
                  _ => {
                     // TODO ERROR
                     eprintln!("Unexpected input {}", line_buf);
                     break;
                  }
               }
            };
            if let Err(e) = res {
               // TODO ERROR
               eprintln!("Encountered I/O error writing in UCI loop: {}", e);
               break;
            }
            line_buf.clear()
         }
         Err(e) => {
            // TODO ERROR
            eprintln!("Encountered I/O error reading in UCI loop: {}", e);
            break;
         }
      }
   }
}
