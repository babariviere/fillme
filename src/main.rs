#[macro_use]
extern crate failure;
extern crate piston_window;

mod filler;

use filler::*;

use failure::Error;
use piston_window::*;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

struct Data {
    board: Vec<Board>,
    pieces: Vec<Piece>,
    players: Vec<Player>,
    end: bool,
}

enum DataKind {
    Piece(Piece),
    Board(Board),
    End,
}

const COLOR_X: [f32; 4] = [0.227, 0.286, 0.643, 1.0];
const COLOR_SX: [f32; 4] = [COLOR_X[0] * 1.5, COLOR_X[1] * 1.5, COLOR_X[2] * 1.5, 1.0];
const COLOR_O: [f32; 4] = [0.937, 0.741, 0.239, 1.0];
const COLOR_SO: [f32; 4] = [COLOR_O[0] * 1.5, COLOR_O[1] * 1.5, COLOR_O[2] * 1.5, 1.0];
const COLOR_NONE: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

const FONT: &'static [u8] = include_bytes!("../assets/font/Roboto-Regular.ttf");

const FONT_SPACE: f64 = 25.0;

struct DisplaySettings {
    stop: bool,
    instant: Instant,
    board_idx: usize,
    scale: f64,
    speed: u32,
    step: i8,
}

fn ui_thread(mut data: Data, rx: mpsc::Receiver<DataKind>) {
    match rx.recv().unwrap() {
        DataKind::Board(b) => data.board.push(b),
        _ => {
            panic!("unexpected haha");
        }
    }
    let size = (
        data.board[0].width() * 10,
        data.board[0].height() * 10 + 100,
    );
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
        speed: 25,
        step: 1,
    };
    while let Some(e) = window.next() {
        if let Some(_r) = e.render_args() {
            let loaded = data.board.len() - 1;
            let b = &data.board[ds.board_idx];
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

                let mut count_p1 = 0;
                let mut count_p2 = 0;
                for (y, row) in b.maps().iter().enumerate() {
                    for (x, col) in row.iter().enumerate() {
                        match *col {
                            'o' | 'O' => count_p1 += 1,
                            'x' | 'X' => count_p2 += 1,
                            _ => {}
                        }
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

                text_o
                    .draw(
                        &format!("{}: {}", data.players[0].name(), count_p1),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(0., FONT_SPACE),
                        g,
                    )
                    .unwrap();

                text_x
                    .draw(
                        &format!("{}: {}", data.players[1].name(), count_p2),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(300., FONT_SPACE),
                        g,
                    )
                    .unwrap();
                text_x
                    .draw(
                        &format!("speed={}%", 100 - ds.speed),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(500., FONT_SPACE),
                        g,
                    )
                    .unwrap();
                text_x
                    .draw(
                        &format!("curr={}, loaded={}", ds.board_idx, loaded),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(700., FONT_SPACE),
                        g,
                    )
                    .unwrap();
            });
        }
        if let Some(array) = e.resize_args() {
            let b = &data.board[ds.board_idx];
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
                    if data.board.len() > (ds.board_idx + 1) {
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
                    if ds.speed <= 95 {
                        ds.speed += 5;
                    }
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
            if elapsed.as_secs() >= 1 || elapsed.subsec_nanos() >= ds.speed * 5_000_000 {
                //println!("total={}, current={}", b.len(), ds.board_idx);
                if ds.step < 0 && ds.board_idx >= (-ds.step) as usize {
                    ds.board_idx -= (-ds.step) as usize;
                } else if data.board.len() > (ds.board_idx + 1) {
                    if ds.step > 0 {
                        ds.board_idx += ds.step as usize;
                    }
                    ds.instant = Instant::now();
                }
            }
        }
        if !data.end {
            while let Ok(d) = rx.try_recv() {
                match d {
                    DataKind::Board(b) => data.board.push(b),
                    DataKind::Piece(p) => data.pieces.push(p),
                    DataKind::End => data.end = true,
                }
            }
        }
    }
}

fn main_loop() -> Result<(), Error> {
    let players = get_players()?;
    let data = Data {
        board: Vec::new(),
        pieces: Vec::new(),
        players: players,
        end: false,
    };
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let buf = read_line_stdin().unwrap();
        if buf.starts_with("Plateau") {
            let board_parsed = Board::read(&buf).unwrap();
            tx.send(DataKind::Board(board_parsed)).unwrap();
        } else if buf.starts_with("Piece") {
            let piece = Piece::read(&buf).unwrap();
            tx.send(DataKind::Piece(piece)).unwrap();
            read_line_stdin().unwrap();
        } else if buf.starts_with("==") {
            let score_o = get_score(buf);
            let score_x = get_score(read_line_stdin().unwrap());
            println!("O={}, X={}", score_o, score_x);
            tx.send(DataKind::End).unwrap();
        } else {
            read_line_stdin().unwrap();
            break;
        }
        //thread::sleep(Duration::from_millis(5));
    });
    ui_thread(data, rx);
    Ok(())
}

fn main() {
    let _ = main_loop().unwrap();
}
