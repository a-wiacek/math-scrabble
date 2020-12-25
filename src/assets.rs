use ggez::graphics;
use ggez::{Context, GameResult};
use std::collections::HashMap;

use crate::board::BoardField;
use crate::tiles::{Digit, MathOperation, Tile, TileStatus};

pub struct Assets {
    pub tile_images: HashMap<Tile, graphics::Image>,
    pub board_field_images: HashMap<BoardField, graphics::Image>,
    pub tile_status_images: HashMap<TileStatus, graphics::Image>,
    pub add_player_image: graphics::Image,
    pub remove_player_image: graphics::Image,
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Assets> {
        let mut tile_images = HashMap::new();
        for d in Digit::all() {
            let digit_image = graphics::Image::new(ctx, format!("/{}.png", d))?;
            tile_images.insert(Tile::Digit(d), digit_image);
        }
        tile_images.insert(
            Tile::Op(MathOperation::Plus),
            graphics::Image::new(ctx, "/plus.png")?,
        );
        tile_images.insert(
            Tile::Op(MathOperation::Minus),
            graphics::Image::new(ctx, "/minus.png")?,
        );
        tile_images.insert(
            Tile::Op(MathOperation::Times),
            graphics::Image::new(ctx, "/times.png")?,
        );
        tile_images.insert(
            Tile::Op(MathOperation::Div),
            graphics::Image::new(ctx, "/div.png")?,
        );
        tile_images.insert(Tile::EqualsSign, graphics::Image::new(ctx, "/equals.png")?);

        let mut board_field_images = HashMap::new();
        board_field_images.insert(
            BoardField::Standard,
            graphics::Image::new(ctx, "/board-field.png")?,
        );
        board_field_images.insert(
            BoardField::DoubleSymbol,
            graphics::Image::new(ctx, "/sym-bonus2.png")?,
        );
        board_field_images.insert(
            BoardField::TripleSymbol,
            graphics::Image::new(ctx, "/sym-bonus3.png")?,
        );
        board_field_images.insert(
            BoardField::DoubleEquation,
            graphics::Image::new(ctx, "/eq-bonus2.png")?,
        );
        board_field_images.insert(
            BoardField::TripleEquation,
            graphics::Image::new(ctx, "/eq-bonus3.png")?,
        );

        let mut tile_status_images = HashMap::new();
        tile_status_images.insert(
            TileStatus::Temporary,
            graphics::Image::new(ctx, "/tile-selected.png")?,
        );
        tile_status_images.insert(
            TileStatus::Permanent,
            graphics::Image::new(ctx, "/tile-base.png")?,
        );

        let add_player_image = graphics::Image::new(ctx, "/add-player.png")?;
        let remove_player_image = graphics::Image::new(ctx, "/remove-player.png")?;
        Ok(Assets {
            tile_images,
            board_field_images,
            tile_status_images,
            add_player_image,
            remove_player_image,
        })
    }
}
