#[macro_use]
extern crate failure;
extern crate piston_window;

mod filler;

use filler::*;

use failure::Error;
use piston_window::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct Shared {
    board: Vec<Board>,
    players: Vec<Player>,
    end: bool,
}

const COLOR_X: [f32; 4] = [0.8, 0.0, 0.0, 1.0];
const COLOR_SX: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const COLOR_O: [f32; 4] = [0.0, 0.8, 0.0, 1.0];
const COLOR_SO: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const COLOR_NONE: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

const FONT: &'static [u8] = include_bytes!("../assets/font/SIMPLIFICA/SIMPLIFICA Typeface.ttf");

const FONT_SPACE: f64 = 25.0;

struct DisplaySettings {
    stop: bool,
    instant: Instant,
    board_idx: usize,
    scale: f64,
    speed: u32,
    step: i8,
}

fn ui_thread(shared: Arc<Mutex<Shared>>) {
    let size;
    loop {
        thread::sleep(Duration::from_millis(100));
        let lock = shared.lock().unwrap();
        let b = &lock.board;
        if b.len() == 0 {
            continue;
        }
        size = (b[0].width() * 10, b[0].height() * 10 + 100);
        break;
    }
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("FillMe", size)
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let factory = window.factory.clone();
    let mut glyphs = Glyphs::from_bytes(FONT, factory, TextureSettings::new()).unwrap();
    let text_o = text::Text::new_color(COLOR_O, 20);
    let text_x = text::Text::new_color(COLOR_X, 20);
    let mut ds = DisplaySettings {
        stop: false,
        instant: Instant::now(),
        board_idx: 0,
        scale: 1.,
        speed: 50,
        step: 1,
    };
    while let Some(e) = window.next() {
        if let Some(_r) = e.render_args() {
            let lock = shared.lock().unwrap();
            let b = &lock.board[ds.board_idx];
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

                text_o
                    .draw(
                        (*lock).players[0].name(),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(0., FONT_SPACE),
                        g,
                    )
                    .unwrap();

                text_x
                    .draw(
                        (*lock).players[1].name(),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(300., FONT_SPACE),
                        g,
                    )
                    .unwrap();

                for (y, row) in b.maps().iter().enumerate() {
                    for (x, col) in row.iter().enumerate() {
                        let color = match *col {
                            'X' => COLOR_X,
                            'x' => COLOR_SX,
                            'O' => COLOR_O,
                            'o' => COLOR_SO,
                            _ => COLOR_NONE,
                        };
                        let rect = [(x * 10) as f64 + 1.0, (y * 10) as f64 + 1.0, 8., 8.];
                        rectangle(
                            color,
                            rect,
                            c.transform.scale(ds.scale, ds.scale).trans(0., FONT_SPACE),
                            g,
                        );
                    }
                }
            });
        }
        if let Some(array) = e.resize_args() {
            let lock = shared.lock().unwrap();
            let b = &lock.board[ds.board_idx];
            let width = (array[0] as f64) / (b.width() as f64 * 10.);
            let height = (array[1] as f64) / ((b.height() as f64 * 10.) + FONT_SPACE);
            if height > width {
                ds.scale = width;
            } else {
                ds.scale = height;
            }
        }
        if let Some(k) = e.press_args() {
            match k {
                Button::Keyboard(keyboard::Key::Space) => ds.stop = !ds.stop,
                Button::Keyboard(keyboard::Key::Left) => {
                    ds.stop = true;
                    if ds.board_idx > 0 {
                        ds.board_idx -= 1;
                    }
                }
                Button::Keyboard(keyboard::Key::Right) => {
                    ds.stop = true;
                    let lock = shared.lock().unwrap();
                    let b = &lock.board;
                    if b.len() > (ds.board_idx + 1) {
                        ds.board_idx += 1;
                    }
                }
                Button::Keyboard(keyboard::Key::R) => {
                    ds.stop = true;
                    ds.board_idx = 0;
                }
                Button::Keyboard(keyboard::Key::Plus) | Button::Keyboard(keyboard::Key::Equals) => {
                    if ds.speed >= 5 {
                        ds.speed -= 5;
                    }
                }
                Button::Keyboard(keyboard::Key::Minus) => {
                    ds.speed += 5;
                }
                Button::Keyboard(keyboard::Key::N) => {
                    ds.step = -1;
                }
                Button::Keyboard(keyboard::Key::M) => {
                    ds.step = 1;
                }
                e => {
                    println!("{:?}", e);
                }
            }
        }
        if !ds.stop {
            let elapsed = ds.instant.elapsed();
            if elapsed.as_secs() >= 1 || elapsed.subsec_nanos() >= ds.speed * 1_000_000 {
                let lock = shared.lock().unwrap();
                let b = &lock.board;
                //println!("total={}, current={}", b.len(), ds.board_idx);
                if ds.step < 0 && ds.board_idx >= (-ds.step) as usize {
                    ds.board_idx -= (-ds.step) as usize;
                } else if b.len() > (ds.board_idx + 1) {
                    if ds.step > 0 {
                        ds.board_idx += ds.step as usize;
                    }
                    ds.instant = Instant::now();
                }
            }
        }
    }
}

fn main_loop() -> Result<(), Error> {
    let players = get_players()?;
    let shared = Arc::new(Mutex::new(Shared {
        board: Vec::new(),
        players: players,
        end: false,
    }));
    let shared_clone = shared.clone();
    thread::spawn(move || loop {
        let buf = read_line_stdin().unwrap();
        if buf.starts_with("Plateau") {
            let board_parsed = Board::read(&buf).unwrap();
            {
                let mut lock = shared.lock().unwrap();
                lock.board.push(board_parsed);
            }
        } else if buf.starts_with("Piece") {
            Piece::read(&buf).unwrap();
            read_line_stdin().unwrap();
        } else if buf.starts_with("==") {
            let score_o = get_score(buf);
            let score_x = get_score(read_line_stdin().unwrap());
            println!("O={}, X={}", score_o, score_x);
            {
                let mut lock = shared.lock().unwrap();
                (*lock).end = true;
            }
        } else {
            read_line_stdin().unwrap();
            break;
        }
        //thread::sleep(Duration::from_millis(5));
    });
    ui_thread(shared_clone);
    Ok(())
}

fn main() {
    let _ = main_loop().unwrap();
}
