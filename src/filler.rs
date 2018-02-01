use failure::Error;
use std::io::{stdin, BufRead};

pub fn read_line_stdin() -> Result<String, Error> {
    let mut buf = String::new();
    let stdin = stdin();
    let mut lock = stdin.lock();
    lock.read_line(&mut buf)?;
    Ok(buf)
}

pub fn get_players() -> Result<Vec<Player>, Error> {
    let mut players = Vec::new();
    read_line_stdin()?;
    read_line_stdin()?;
    read_line_stdin()?;
    read_line_stdin()?;
    read_line_stdin()?;
    read_line_stdin()?;
    players.push(Player::from_line(read_line_stdin()?));
    read_line_stdin()?;
    players.push(Player::from_line(read_line_stdin()?));
    Ok(players)
}

pub fn get_score(line: String) -> u32 {
    line.chars()
        .filter(|c| c.is_numeric())
        .collect::<String>()
        .parse()
        .unwrap()
}

#[derive(Debug)]
enum PlayerSymbol {
    X,
    O,
}

#[derive(Debug)]
pub struct Player {
    name: String,
    sym: PlayerSymbol,
}

impl Player {
    pub fn from_line(line: String) -> Player {
        let c = line.chars().nth(10);
        let c = if c == Some('1') {
            PlayerSymbol::O
        } else {
            PlayerSymbol::X
        };
        let end_idx = line.rfind(".filler").unwrap_or(line.rfind("]").unwrap());
        let beg_idx = line.find('[').unwrap();
        let name = match line.rfind('/') {
            Some(idx) => &line[idx + 1..end_idx],
            None => &line[beg_idx + 1..end_idx],
        }.to_owned();
        Player { name: name, sym: c }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct Board {
    width: u32,
    height: u32,
    maps: Vec<Vec<char>>,
}

impl Board {
    pub fn read(line: &str) -> Result<Board, Error> {
        let splitted = line.split_whitespace()
            .map(|s| s.chars().filter(|c| c.is_numeric()).collect::<String>())
            .collect::<Vec<String>>();
        if splitted.len() < 2 {
            bail!("eof");
        }
        let width = splitted[2].parse()?;
        let height = splitted[1].parse()?;
        read_line_stdin()?;
        let mut maps = Vec::new();
        for _ in 0..height {
            let line = read_line_stdin()?;
            maps.push(
                line.chars()
                    .filter(|c| *c == '.' || *c == 'x' || *c == 'X' || *c == 'o' || *c == 'O')
                    .collect::<Vec<char>>(),
            );
        }
        Ok(Board {
            width: width,
            height: height,
            maps: maps,
        })
    }

    pub fn maps(&self) -> &Vec<Vec<char>> {
        &self.maps
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Piece {
    width: u32,
    height: u32,
    piece: Vec<Vec<char>>,
}

impl Piece {
    pub fn read(line: &str) -> Result<Piece, Error> {
        let splitted = line.split_whitespace()
            .map(|s| s.chars().filter(|c| c.is_numeric()).collect::<String>())
            .collect::<Vec<String>>();
        if splitted.len() < 2 {
            bail!("eof");
        }
        let x = splitted[2].parse()?;
        let y = splitted[1].parse()?;
        let mut piece = Vec::new();
        for _ in 0..y {
            let line = read_line_stdin()?;
            piece.push(
                line.chars()
                    .filter(|c| *c == '.' || *c == 'x' || *c == 'X' || *c == 'o' || *c == 'O')
                    .collect::<Vec<char>>(),
            );
        }
        Ok(Piece {
            width: x,
            height: y,
            piece: piece,
        })
    }
}
