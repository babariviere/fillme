#[macro_use]
extern crate failure;
extern crate sdl2;

mod filler;

use filler::*;

use failure::Error;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::WindowPos;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct FontRenderer<'ttf, 'r> {
    font: Arc<Font<'ttf, 'r>>,
    color: Color,
    textures: HashMap<char, Texture<'r>>,
}

impl<'ttf, 'r> FontRenderer<'ttf, 'r> {
    pub fn new(font: Font<'ttf, 'r>, color: Color) -> FontRenderer<'ttf, 'r> {
        FontRenderer {
            font: Arc::new(font),
            color: color,
            textures: HashMap::new(),
        }
    }

    pub fn render<T: RenderTarget, C>(
        &mut self,
        canvas: &mut Canvas<T>,
        texture_creator: &'r TextureCreator<C>,
        s: &str,
        dst: Point,
    ) {
        //
        let len = s.len();
        let (w, h) = self.font.size_of(s).unwrap();
        let mut dst = Rect::new(dst.x(), dst.y(), (w / len as u32), h);
        let font_clone = self.font.clone();
        let color_clone = self.color.clone();
        for c in s.chars() {
            let tex = self.textures.entry(c).or_insert_with(|| {
                let surface = font_clone
                    .render(format!("{}", c).as_str())
                    .blended(color_clone)
                    .unwrap();
                texture_creator
                    .create_texture_from_surface(&surface)
                    .unwrap()
            });
            canvas.copy(&tex, None, dst).unwrap();
            let nx = dst.x() + dst.width() as i32;
            dst.set_x(nx);
        }
    }
}

struct Shared {
    board: Board,
    players: Vec<Player>,
}

fn ui_thread(board: Arc<Mutex<Board>>) {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut window = video_subsystem
        .window("FillMe", 800, 600)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();
    loop {
        thread::sleep(Duration::from_millis(100));
        let board = board.lock().unwrap();
        if board.width() == 0 {
            continue;
        }
        let (nw, nh) = (board.width() * 10, board.height() * 10);
        window.set_size(nw, nh).unwrap();
        window.set_position(WindowPos::Centered, WindowPos::Centered);
        break;
    }
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut font = ttf_context
        .load_font("/System/Library/Fonts/Menlo.ttc", 32)
        .unwrap();
    let mut font_renderer_p1 = FontRenderer::new(font, Color::RGB(180, 0, 0));
    let font = ttf_context
        .load_font("/System/Library/Fonts/Menlo.ttc", 25)
        .unwrap();
    let mut font_renderer_p2 = FontRenderer::new(font, Color::RGB(0, 180, 0));

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(75, 75, 75));
        canvas.clear();
        let mut row = 0;
        let mut col = 0;
        for (row_idx, row) in board.lock().unwrap().maps().iter().enumerate() {
            for (col_idx, col) in row.iter().enumerate() {
                match *col {
                    'X' => canvas.set_draw_color(Color::RGB(180, 0, 0)),
                    'x' => canvas.set_draw_color(Color::RGB(255, 0, 0)),
                    'O' => canvas.set_draw_color(Color::RGB(0, 180, 0)),
                    'o' => canvas.set_draw_color(Color::RGB(0, 255, 0)),
                    _ => canvas.set_draw_color(Color::RGB(180, 180, 180)),
                }
                canvas
                    .fill_rect(Rect::new(
                        10 * col_idx as i32 + 1,
                        10 * row_idx as i32 + 1,
                        8,
                        8,
                    ))
                    .unwrap();
            }
        }
        font_renderer_p1.render(
            &mut canvas,
            &texture_creator,
            "hello world!",
            Point::new(10, 10),
        );
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn main_loop() -> Result<(), Error> {
    let players = get_players()?;
    let board = Arc::new(Mutex::new(Board::empty()));
    let shared = Arc::new(Mutex::new(Shared {
        board: Board::empty(),
        players: players,
    }));
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
