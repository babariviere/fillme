#[macro_use]
extern crate failure;
extern crate piston_window;

mod filler;

use filler::*;

use failure::Error;
use piston_window::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

//struct Shared {
//    board: Vec<Board>,
//    players: Vec<Player>,
//}

const COLOR_X: [f32; 4] = [0.8, 0.0, 0.0, 1.0];
const COLOR_SX: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const COLOR_O: [f32; 4] = [0.0, 0.8, 0.0, 1.0];
const COLOR_SO: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const COLOR_NONE: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

const FONT: &'static [u8] = include_bytes!("../assets/font/SIMPLIFICA/SIMPLIFICA Typeface.ttf");

fn ui_thread(board: Arc<Mutex<Board>>) {
    let size;
    loop {
        thread::sleep(Duration::from_millis(100));
        let board = board.lock().unwrap();
        if board.width() == 0 {
            continue;
        }
        size = (board.width() * 10, board.height() * 10);
        break;
    }
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("FillMe", size)
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut scale = 1.;
    while let Some(e) = window.next() {
        if let Some(r) = e.render_args() {
            let b = board.lock().unwrap();
            window.draw_2d(&e, |c, g| {
                clear(
                    [
                        COLOR_NONE[0] - 0.1,
                        COLOR_NONE[1] - 0.1,
                        COLOR_NONE[2] - 0.1,
                        1.0,
                    ],
                    g,
                );

                for (y, row) in b.maps().iter().enumerate() {
                    for (x, col) in row.iter().enumerate() {
                        let color = match *col {
                            'X' => COLOR_X,
                            'x' => COLOR_SX,
                            'O' => COLOR_O,
                            'o' => COLOR_SO,
                            _ => COLOR_NONE,
                        };
                        let rect = [(x * 10) as f64, (y * 10) as f64, 10., 10.].margin(1.0);
                        rectangle(color, rect, c.transform.scale(scale, scale), g);
                    }
                }
            });
        } else if let Some(array) = e.resize_args() {
            let b = board.lock().unwrap();
            let width = array[0] as f64 / (b.width() as f64 * 10.);
            let height = array[1] as f64 / (b.height() as f64 * 10.);
            if height > width {
                scale = width;
            } else {
                scale = height;
            }
        }
    }
}

fn main_loop() -> Result<(), Error> {
    let players = get_players()?;
    let board = Arc::new(Mutex::new(Board::empty()));
    //let shared = Arc::new(Mutex::new(Shared {
    //    board: Board::empty(),
    //    players: players,
    //}));
    let board_clone = board.clone();
    thread::spawn(move || loop {
        let buf = read_line_stdin().unwrap();
        if buf.starts_with("Plateau") {
            let board_parsed = Board::read(buf).unwrap();
            {
                let mut lock = board.lock().unwrap();
                *lock = board_parsed;
            }
        } else if buf.starts_with("Piece") {
            Piece::read(buf).unwrap();
            read_line_stdin().unwrap();
        } else if buf.starts_with("==") {
            let score_o = get_score(buf);
            let score_x = get_score(read_line_stdin().unwrap());
            println!("O={}, X={}", score_o, score_x);
        } else {
            read_line_stdin().unwrap();
            break;
        }
        thread::sleep(Duration::from_millis(5));
    });
    ui_thread(board_clone);
    Ok(())
}

fn main() {
    let _ = main_loop().unwrap();
}
