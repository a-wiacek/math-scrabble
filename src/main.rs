mod active_game;
mod assets;
mod board;
mod consts;
mod game;
mod highscores;
mod lobby;
mod summary;
mod tiles;

use crate::game::Game;
use ggez::event;
use ggez::{conf, ContextBuilder, GameResult};
use std::{env, path};

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    // Set up logger
    fern::Dispatch::new()
        // Format logs
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{:<5}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level().to_string(),
                record.target(),
                message
            ))
        })
        // `gfx_device_gl` is very chatty on info loglevel, so
        // filter that a bit more strictly.
        .level_for("gfx_device_gl", log::LevelFilter::Warn)
        .level(log::LevelFilter::Trace)
        // Hooks up console output.
        .chain(std::io::stderr())
        .apply()
        .unwrap();

    let cb = ContextBuilder::new("math-scrabble", "a-wiacek")
        .window_setup(
            conf::WindowSetup::default()
                .title("Math scrabble")
                .icon("/equals.png"),
        )
        .window_mode(
            conf::WindowMode::default().dimensions(consts::WINDOW_SIZE.0, consts::WINDOW_SIZE.1),
        )
        .add_resource_path(resource_dir);

    let (ctx, events_loop) = &mut cb.build()?;

    let game = &mut Game::new(ctx)?;
    event::run(ctx, events_loop, game)
}

#[cfg(test)]
mod tests {
    use crate::board::{eval_expression, InvalidEquation};
    use crate::tiles::{Digit, MathOperation, Tile};
    use num::rational::Ratio;

    fn parse_and_eval_expression(text: &str) -> Result<Ratio<i64>, InvalidEquation> {
        eval_expression(
            text.chars()
                .map(|c| match c {
                    '+' => Tile::Op(MathOperation::Plus),
                    '-' => Tile::Op(MathOperation::Minus),
                    '*' => Tile::Op(MathOperation::Times),
                    '/' => Tile::Op(MathOperation::Div),
                    '0' => Tile::Digit(Digit::D0),
                    '1' => Tile::Digit(Digit::D1),
                    '2' => Tile::Digit(Digit::D2),
                    '3' => Tile::Digit(Digit::D3),
                    '4' => Tile::Digit(Digit::D4),
                    '5' => Tile::Digit(Digit::D5),
                    '6' => Tile::Digit(Digit::D6),
                    '7' => Tile::Digit(Digit::D7),
                    '8' => Tile::Digit(Digit::D8),
                    '9' => Tile::Digit(Digit::D9),
                    _ => panic!(),
                })
                .collect::<Vec<Tile>>()
                .as_slice(),
        )
    }

    #[test]
    fn test_eval_expressions() {
        assert_eq!(
            parse_and_eval_expression("2*2+2"),
            Ok(Ratio::new(6, 1)),
            "2*2+2"
        );
        assert_eq!(
            parse_and_eval_expression("2+2*2"),
            Ok(Ratio::new(6, 1)),
            "2+2*2"
        );
        assert_eq!(
            parse_and_eval_expression("2-3-4"),
            Ok(Ratio::new(-5, 1)),
            "2-3-4"
        );
        assert_eq!(
            parse_and_eval_expression("2-3+4"),
            Ok(Ratio::new(3, 1)),
            "2-3+4"
        );
        assert_eq!(
            parse_and_eval_expression("2+3-4"),
            Ok(Ratio::new(1, 1)),
            "2+3-4"
        );
        assert_eq!(
            parse_and_eval_expression("2*2+3*3"),
            Ok(Ratio::new(13, 1)),
            "2*2+3*3"
        );
        assert_eq!(
            parse_and_eval_expression("2+2*3+3"),
            Ok(Ratio::new(11, 1)),
            "2+2*3+3"
        );
        assert_eq!(
            parse_and_eval_expression("9/0"),
            Err(InvalidEquation::DivByZero),
            "9/0"
        );
        assert_eq!(
            parse_and_eval_expression("/3"),
            Err(InvalidEquation::ParsingError),
            "/3"
        );
        assert_eq!(
            parse_and_eval_expression("01/3"),
            Err(InvalidEquation::LeadingZero),
            "01/3"
        );
    }
}
