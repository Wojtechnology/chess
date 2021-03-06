use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use serde::Serialize;
use serde_json::json;

mod piece {
    use super::{Board, Location, WalkStrategy};

    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Type {
        Pawn,
        Bishop,
        Knight,
        Rook,
        Queen,
        King,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Color {
        White,
        Black,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Piece {
        pub tpe: Type,
        pub color: Color,
    }

    impl Piece {
        pub fn new(tpe: Type, color: Color) -> Piece {
            Piece {
                tpe: tpe,
                color: color,
            }
        }

        pub fn new_opt(tpe: Type, color: Color) -> Option<Piece> {
            Some(Self::new(tpe, color))
        }

        fn strategies_pawn(&self, from: Location) -> Vec<WalkStrategy> {
            match self.color {
                Color::White => {
                    if from.y == 1 {
                        vec![WalkStrategy::new(0, 1, 2)]
                    } else {
                        vec![WalkStrategy::new(0, 1, 1)]
                    }
                }
                Color::Black => {
                    if from.y == 6 {
                        vec![WalkStrategy::new(0, -1, 2)]
                    } else {
                        vec![WalkStrategy::new(0, -1, 1)]
                    }
                }
            }
        }

        pub fn valid_moves(&self, board: &Board, from: Location) -> Vec<Location> {
            let strategies = match self.tpe {
                Type::Pawn => self.strategies_pawn(from),
                Type::Bishop => vec![
                    WalkStrategy::new(-1, -1, 7),
                    WalkStrategy::new(-1, 1, 7),
                    WalkStrategy::new(1, -1, 7),
                    WalkStrategy::new(1, 1, 7),
                ],
                Type::Knight => vec![
                    WalkStrategy::new(-2, -1, 1),
                    WalkStrategy::new(-2, 1, 1),
                    WalkStrategy::new(-1, -2, 1),
                    WalkStrategy::new(-1, 2, 1),
                    WalkStrategy::new(1, -2, 1),
                    WalkStrategy::new(1, 2, 1),
                    WalkStrategy::new(2, -1, 1),
                    WalkStrategy::new(2, 1, 1),
                ],
                Type::Rook => vec![
                    WalkStrategy::new(-1, 0, 7),
                    WalkStrategy::new(0, -1, 7),
                    WalkStrategy::new(0, 1, 7),
                    WalkStrategy::new(1, 0, 7),
                ],
                Type::Queen => vec![
                    WalkStrategy::new(-1, -1, 7),
                    WalkStrategy::new(-1, 1, 7),
                    WalkStrategy::new(1, -1, 7),
                    WalkStrategy::new(1, 1, 7),
                    WalkStrategy::new(-1, 0, 7),
                    WalkStrategy::new(0, -1, 7),
                    WalkStrategy::new(0, 1, 7),
                    WalkStrategy::new(1, 0, 7),
                ],
                Type::King => vec![
                    WalkStrategy::new(-1, -1, 1),
                    WalkStrategy::new(-1, 1, 1),
                    WalkStrategy::new(1, -1, 1),
                    WalkStrategy::new(1, 1, 1),
                    WalkStrategy::new(-1, 0, 1),
                    WalkStrategy::new(0, -1, 1),
                    WalkStrategy::new(0, 1, 1),
                    WalkStrategy::new(1, 0, 1),
                ],
            };
            let mut moves = Vec::new();
            for strategy in strategies {
                let walk = strategy.to_walk(from);
                for dest in walk {
                    if let Some(piece) = board.0[dest.y as usize][dest.x as usize] {
                        if piece.color == self.color {
                            break;
                        }
                    }
                    moves.push(dest)
                }
            }
            moves
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Location {
    x: u8,
    y: u8,
}

impl Location {
    pub fn to_string(&self) -> String {
        format!("{}{}", (self.x + 97) as char, self.y + 1)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Copy, Clone)]
struct WalkStrategy {
    dx: i8,
    dy: i8,
    max_steps: u8,
}

impl WalkStrategy {
    pub fn new(dx: i8, dy: i8, max_steps: u8) -> WalkStrategy {
        WalkStrategy {
            dx: dx,
            dy: dy,
            max_steps: max_steps,
        }
    }

    pub fn to_walk(&self, start: Location) -> Walk {
        Walk {
            dx: self.dx,
            dy: self.dy,
            steps_left: self.max_steps,
            cur: start,
        }
    }
}

struct Walk {
    dx: i8,
    dy: i8,
    steps_left: u8,
    cur: Location,
}

impl Iterator for Walk {
    type Item = Location;

    fn next(&mut self) -> Option<Location> {
        if self.steps_left == 0 {
            None
        } else if self.dx < 0 && -self.dx as u8 > self.cur.x {
            None
        } else if self.dx > 0 && self.dx as u8 + self.cur.x >= 8 {
            None
        } else if self.dy < 0 && -self.dy as u8 > self.cur.y {
            None
        } else if self.dy > 0 && self.dy as u8 + self.cur.y >= 8 {
            None
        } else {
            self.steps_left -= 1;
            self.cur.x = (self.cur.x as i8 + self.dx) as u8;
            self.cur.y = (self.cur.y as i8 + self.dy) as u8;
            Some(self.cur)
        }
    }
}

struct Board([[Option<piece::Piece>; 8]; 8]);

impl Board {
    pub fn new() -> Board {
        use piece::{Color, Piece, Type};
        Board([
            [
                Piece::new_opt(Type::Rook, Color::White),
                Piece::new_opt(Type::Knight, Color::White),
                Piece::new_opt(Type::Bishop, Color::White),
                Piece::new_opt(Type::Queen, Color::White),
                Piece::new_opt(Type::King, Color::White),
                Piece::new_opt(Type::Bishop, Color::White),
                Piece::new_opt(Type::Knight, Color::White),
                Piece::new_opt(Type::Rook, Color::White),
            ],
            [
                Piece::new_opt(Type::Pawn, Color::White),
                Piece::new_opt(Type::Pawn, Color::White),
                Piece::new_opt(Type::Pawn, Color::White),
                Piece::new_opt(Type::Pawn, Color::White),
                Piece::new_opt(Type::Pawn, Color::White),
                Piece::new_opt(Type::Pawn, Color::White),
                Piece::new_opt(Type::Pawn, Color::White),
                Piece::new_opt(Type::Pawn, Color::White),
            ],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [
                Piece::new_opt(Type::Pawn, Color::Black),
                Piece::new_opt(Type::Pawn, Color::Black),
                Piece::new_opt(Type::Pawn, Color::Black),
                Piece::new_opt(Type::Pawn, Color::Black),
                Piece::new_opt(Type::Pawn, Color::Black),
                Piece::new_opt(Type::Pawn, Color::Black),
                Piece::new_opt(Type::Pawn, Color::Black),
                Piece::new_opt(Type::Pawn, Color::Black),
            ],
            [
                Piece::new_opt(Type::Rook, Color::Black),
                Piece::new_opt(Type::Knight, Color::Black),
                Piece::new_opt(Type::Bishop, Color::Black),
                Piece::new_opt(Type::Queen, Color::Black),
                Piece::new_opt(Type::King, Color::Black),
                Piece::new_opt(Type::Bishop, Color::Black),
                Piece::new_opt(Type::Knight, Color::Black),
                Piece::new_opt(Type::Rook, Color::Black),
            ],
        ])
    }

    pub fn step(&mut self, from: Location, to: Location) -> Result<(), String> {
        let piece = match self.0[from.y as usize][from.x as usize] {
            None => Err(format!("No piece at {}", from)),
            Some(p) => Ok(p),
        }?;
        let valid_moves = piece.valid_moves(self, from);
        let () = if valid_moves.iter().any(|&dest| dest == to) {
            Ok(())
        } else {
            Err("Invalid move".to_string())
        }?;
        self.0[from.y as usize][from.x as usize] = None;
        self.0[to.y as usize][to.x as usize] = Some(piece);
        Ok(())
    }
}

fn cell_as_str(cell: &Option<piece::Piece>) -> String {
    use piece::{Color, Piece, Type};
    match cell {
        None => "".to_string(),
        Some(Piece { tpe, color }) => {
            let c = match color {
                Color::White => "w",
                Color::Black => "b",
            };
            let t = match tpe {
                Type::Pawn => "P",
                Type::Bishop => "B",
                Type::Knight => "N",
                Type::Rook => "R",
                Type::Queen => "Q",
                Type::King => "K",
            };
            format!("{}{}", c, t)
        }
    }
}

fn board_as_str(board: &Board) -> String {
    let mut cells = Vec::with_capacity(64);
    for i in 0..8 {
        for j in 0..8 {
            cells.push(cell_as_str(&board.0[i][j]));
        }
    }
    cells.join(",")
}

fn get_path(mut stream: &TcpStream) -> (String, HashMap<String, String>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let req_str = String::from_utf8_lossy(&buffer[..]);
    let req_fst_line = req_str.split('\n').next().unwrap();
    let mut req_fst_line_it = req_fst_line.split(' ');
    req_fst_line_it.next().unwrap(); // Method
    let full_path = req_fst_line_it.next().unwrap();
    let mut full_path_it = full_path.split("?");
    let path = full_path_it.next().unwrap().to_string();
    let query_str_it = {
        match full_path_it.next() {
            Some(query_str) => query_str.split("&"),
            None => {
                // Make the split empty
                let mut split = "".split("&");
                split.next().unwrap();
                split
            }
        }
    };
    let mut query_args = HashMap::new();
    for query_arg_str in query_str_it {
        let mut query_arg_str_it = query_arg_str.split("=");
        query_args.insert(
            query_arg_str_it.next().unwrap().to_string(),
            query_arg_str_it.collect::<Vec<&str>>().join("="),
        );
    }
    (path, query_args)
}

fn location_from_string(s: &String) -> Location {
    let i = s.parse::<u8>().unwrap();
    Location { x: i % 8, y: i / 8 }
}

fn get_from_to(query_args: HashMap<String, String>) -> (Location, Location) {
    let from_raw = query_args.get("from").unwrap();
    let to_raw = query_args.get("to").unwrap();
    (location_from_string(from_raw), location_from_string(to_raw))
}

#[derive(Serialize)]
struct ResponseData {
    squares: String,
}

fn success_res(content: String) -> String {
    format!(
        "\
HTTP/1.1 200 OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Content-Type: application/json\r\n\
Content-Length: {}\r\n\
\r\n\
{}",
        content.len(),
        content,
    )
}

fn bad_request_res(err_msg: String) -> String {
    format!(
        "\
HTTP/1.1 400 Bad Request\r\n\
Access-Control-Allow-Origin: *\r\n\
Content-Type: text/plain\r\n\
Content-Length: {}\r\n\
\r\n\
{}",
        err_msg.len(),
        err_msg,
    )
}

fn write_board(board: &Board, mut stream: &TcpStream) {
    let data = ResponseData {
        squares: board_as_str(board),
    };
    let body = json!(data).to_string();
    let response = success_res(body);
    stream.write(response.as_bytes()).unwrap();
}

fn write_err(err_msg: String, mut stream: &TcpStream) {
    let response = bad_request_res(err_msg);
    stream.write(response.as_bytes()).unwrap();
}

fn main() {
    let mut board = Board::new();
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let (path, query_args) = get_path(&stream);
        println!("{}: {:?}", path, query_args);
        if path.eq("/game") {
            write_board(&board, &stream);
        } else if path.eq("/move") {
            let (from, to) = get_from_to(query_args);
            match board.step(from, to) {
                Ok(()) => write_board(&board, &stream),
                Err(e) => {
                    println!("Error: {}", e);
                    write_err(e, &stream)
                }
            };
        } else {
            // TODO: 404
            write_err("Unknown path".to_string(), &stream);
        }
        stream.flush().unwrap()
    }
}
