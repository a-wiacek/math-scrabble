use ggez::graphics;
use ggez::{Context, GameResult};
use std::collections::HashMap;

use crate::active_game::Player;
use crate::assets::Assets;
use crate::board::Board;
use crate::consts;
use crate::tiles::{MathOperation, Tile, TileOnBoard, TileStatus};

pub struct NewGameScreen {
    players: Vec<String>,
}

pub struct Summary {
    players: Vec<Player>,
}

pub enum GameStatus {
    NewGameScreen(NewGameScreen),
    ActiveGame(ActiveGame),
    Summary(Summary),
}

pub struct Game {
    game: GameStatus,
    assets: Assets,
}

impl Game {
    pub fn new(ctx: &mut Context) -> GameResult<Game> {
        let game = GameStatus::NewGameScreen(NewGameScreen { players: vec![] });
        let assets = Assets::new(ctx)?;
        Ok(Game { game, assets })
    }
}
