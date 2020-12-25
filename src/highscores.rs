use crate::consts::{HIGHSCORES_FILE, HIGHSCORES_STORED};
use ggez::{filesystem, Context, GameResult};
use std::io::{Read, Write};

// Top 10 scores are kept in highscores. They are not
// distinguished by number and type of players in game,
// but AI scores are not kept.

#[derive(Clone)]
pub struct HighscoresEntry {
    pub name: String,
    pub score: u64,
    new: bool,
}

impl HighscoresEntry {
    pub fn is_new(&self) -> bool {
        self.new
    }
}

pub struct Highscores {
    entries: Vec<HighscoresEntry>, // length at most HIGHSCORES_STORED
}

impl Highscores {
    // Reads scores kept or, if file was not created yet,
    // creates an empty structure.
    // File structure: 2n lines, each pair of lines contains 1: name, 2: score
    // File is not encrypted in any way
    // If file is corrupted, remove it
    pub fn new(ctx: &mut Context) -> GameResult<Highscores> {
        enum ParseResult {
            Successful(Highscores),
            FileDoesNotExist,
            CorruptedFile,
        }
        fn parse(ctx: &mut Context) -> ParseResult {
            if !filesystem::exists(ctx, HIGHSCORES_FILE) {
                return ParseResult::FileDoesNotExist;
            }
            let mut file = filesystem::open(ctx, HIGHSCORES_FILE).unwrap();
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Ok(_) => {}
                Err(_) => return ParseResult::CorruptedFile,
            }
            let lines: Vec<&str> = content.lines().filter(|line| !line.is_empty()).collect();
            if lines.len() > 2 * HIGHSCORES_STORED || lines.len() % 2 == 1 {
                return ParseResult::CorruptedFile;
            }
            match lines
                .chunks(2)
                .map(
                    |entry| -> Result<HighscoresEntry, std::num::ParseIntError> {
                        let player_name = entry[0].to_string();
                        let score = entry[1].parse::<u64>()?;
                        Ok(HighscoresEntry {
                            new: false,
                            name: player_name,
                            score,
                        })
                    },
                )
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(entries) => {
                    let descending = entries
                        .windows(2)
                        .all(|window| window[0].score >= window[1].score);
                    if descending {
                        ParseResult::Successful(Highscores { entries })
                    } else {
                        ParseResult::CorruptedFile
                    }
                }
                Err(_) => ParseResult::CorruptedFile,
            }
        }
        match parse(ctx) {
            ParseResult::Successful(highscores) => Ok(highscores),
            ParseResult::FileDoesNotExist => Ok(Highscores { entries: vec![] }),
            ParseResult::CorruptedFile => {
                filesystem::delete(ctx, HIGHSCORES_FILE)?;
                Ok(Highscores { entries: vec![] })
            }
        }
    }

    pub fn entries(&self) -> &[HighscoresEntry] {
        &self.entries
    }

    // If the score was good enough, updates the structure and returns true.
    // Otherwise, return false.
    pub fn update(&mut self, player_name: String, score: u64) -> bool {
        let changed = score > 0
            && (score > self.entries.last().map(|entry| entry.score).unwrap_or(0)
                || self.entries.len() < HIGHSCORES_STORED);
        if changed {
            let index = self
                .entries
                .iter()
                .position(|entry| entry.score < score)
                .unwrap_or_else(|| self.entries.len());
            self.entries.insert(
                index,
                HighscoresEntry {
                    name: player_name,
                    score,
                    new: true,
                },
            );
            if self.entries.len() > HIGHSCORES_STORED {
                self.entries.pop();
            }
        }
        changed
    }

    // Save highscores in file.
    pub fn save(self, ctx: &mut Context) -> GameResult {
        let mut file = filesystem::create(ctx, HIGHSCORES_FILE)?;
        let input = self
            .entries
            .into_iter()
            .map(|entry| format!("{}\n{}\n", entry.name, entry.score))
            .collect::<String>();
        file.write_all(input.as_bytes())?;
        Ok(())
    }
}
