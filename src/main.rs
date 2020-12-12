use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;

use serde::Serialize;
use serde_json::json;

mod piece {
    pub enum Type {
        Pawn,
        Bishop,
        Knight,
        Rook,
        Queen,
        King,
    }

    pub enum Color {
        White,
        Black,
    }

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
    }
}

struct Location {
    x: u8,
    y: u8,
}

struct Move {
    piece: piece::Piece,
    location: Location,
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

fn get_path(mut stream: &TcpStream) -> String {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let req_str = String::from_utf8_lossy(&buffer[..]);
    let req_fst_line = req_str.split('\n').next().unwrap();
    let mut req_fst_line_it = req_fst_line.split(' ');
    req_fst_line_it.next().unwrap(); // Method
    req_fst_line_it.next().unwrap().to_string() // Path
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
{}
",
        content.len(),
        content
    )
}

fn bad_request_res() -> String {
    return String::from_str("HTTP/1.1 400 Bad Request\r\nAccess-Control-Allow-Origin: *\r\n")
        .unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let board = Board::new();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let path = get_path(&stream);
        if path.eq("/game") {
            let data = ResponseData {
                squares: board_as_str(&board),
            };
            let body = json!(data).to_string();
            let response = success_res(body);
            stream.write(response.as_bytes()).unwrap();
        } else {
            let response = bad_request_res();
            stream.write(response.as_bytes()).unwrap();
        }
        stream.flush().unwrap()
    }
}
