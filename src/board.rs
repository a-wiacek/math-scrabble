use num::rational::Ratio;
use std::collections::HashSet;
use std::fmt;

use crate::assets::Assets;
use crate::consts;
use crate::tiles::{Digit, MathOperation, Tile, TileOnBoard, TileStatus};
use ggez::{
    graphics::{spritebatch::SpriteBatch, DrawParam, Drawable},
    Context, GameResult,
};

fn neighbours(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut vec = Vec::new();
    if x > 0 {
        vec.push((x - 1, y));
    }
    if x < consts::BOARD_SIZE - 1 {
        vec.push((x + 1, y));
    }
    if y > 0 {
        vec.push((x, y - 1));
    }
    if y < consts::BOARD_SIZE - 1 {
        vec.push((x, y + 1));
    }
    vec
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BoardField {
    Standard,
    DoubleSymbol,
    TripleSymbol,
    DoubleEquation,
    TripleEquation,
}

impl BoardField {
    fn symbol_multiplier(&self) -> u64 {
        match self {
            BoardField::DoubleSymbol => 2,
            BoardField::TripleSymbol => 3,
            _ => 1,
        }
    }

    fn equation_multiplier(&self) -> u64 {
        match self {
            BoardField::DoubleEquation => 2,
            BoardField::TripleEquation => 3,
            _ => 1,
        }
    }
}

enum BoardLine {
    Row(usize),
    Column(usize),
}

#[derive(Copy, Clone)]
enum EquationDirection {
    Horizontal,
    Vertical,
}

impl std::ops::Neg for EquationDirection {
    type Output = EquationDirection;

    fn neg(self) -> Self::Output {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

impl fmt::Display for EquationDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Horizontal => write!(f, "Horizontal"),
            Self::Vertical => write!(f, "Vertical"),
        }
    }
}

pub struct EquationToVerify {
    initial_position: (usize, usize),
    direction: EquationDirection,
    tiles: Vec<Tile>,
}

impl fmt::Display for EquationToVerify {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} equation starting at ({}, {}) [{}]",
            self.direction,
            self.initial_position.0 + 1,
            self.initial_position.1 + 1,
            self.tiles
                .iter()
                .map(|tile| tile.to_string())
                .collect::<Vec<String>>()
                .join("")
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidEquation {
    OperatorsWithoutEqualsSign,
    MoreThanOneEqualsSign,
    LeadingZero,
    ParsingError,
    DivByZero,
    SidesNotEqual(Ratio<i64>, Ratio<i64>),
}

impl fmt::Display for InvalidEquation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperatorsWithoutEqualsSign => write!(
                f,
                "Equations without equals sign are allowed only to have digits."
            ),
            Self::MoreThanOneEqualsSign => {
                write!(f, "The equation contains more than one equals sign.")
            }
            Self::LeadingZero => write!(
                f,
                "The equation contains a number starting with leading zero."
            ),
            Self::ParsingError => write!(f, "Could not parse the equation."),
            Self::DivByZero => write!(
                f,
                "Division by zero encountered while evaluating the equation."
            ),
            Self::SidesNotEqual(lhs, rhs) => write!(
                f,
                "Left hand side is equal to {} and right hand side is equal to {}.",
                lhs, rhs
            ),
        }
    }
}

// Implementation of shunting-yard algorithm. The vector of tiles should not have equals sign in it.
// TODO: This function is very testable - write tests!
pub fn eval_expression(tiles: &[Tile]) -> Result<Ratio<i64>, InvalidEquation> {
    let mut output_stack = Vec::<Ratio<i64>>::new();
    let mut operator_stack = Vec::<MathOperation>::new();
    let mut curr_number: Option<i64> = None;
    for tile in tiles {
        match tile {
            &Tile::Digit(d) => {
                if curr_number == Some(0) {
                    return Err(InvalidEquation::LeadingZero);
                }
                let r = curr_number.get_or_insert(0);
                *r = 10 * *r + d.value();
            }
            Tile::Op(op) => {
                match curr_number {
                    None => return Err(InvalidEquation::ParsingError),
                    Some(n) => output_stack.push(Ratio::from_integer(n)),
                }
                curr_number = None;
                while operator_stack
                    .last()
                    .map(|stack_op| stack_op.precedence() <= op.precedence())
                    .unwrap_or(false)
                {
                    let stack_op = operator_stack.pop().unwrap();
                    if output_stack.len() < 2 {
                        return Err(InvalidEquation::ParsingError);
                    } else {
                        let b = output_stack.pop().unwrap();
                        let a = output_stack.pop().unwrap();
                        match stack_op.eval(a, b) {
                            None => return Err(InvalidEquation::DivByZero),
                            Some(c) => output_stack.push(c),
                        }
                    }
                }
                operator_stack.push(*op);
            }
            Tile::EqualsSign => panic!("parse_expression: Tile::EqualsSign"),
        }
    }

    match curr_number {
        None => return Err(InvalidEquation::ParsingError),
        Some(n) => output_stack.push(Ratio::from_integer(n)),
    }
    if output_stack.len() != operator_stack.len() + 1 {
        return Err(InvalidEquation::ParsingError);
    }
    operator_stack.reverse();
    for op in operator_stack {
        let b = output_stack.pop().unwrap();
        let a = output_stack.pop().unwrap();
        match op.eval(a, b) {
            None => return Err(InvalidEquation::DivByZero),
            Some(c) => output_stack.push(c),
        }
    }

    Ok(output_stack[0])
}

impl EquationToVerify {
    fn has_equals_sign(&self) -> bool {
        self.tiles.iter().any(|t| matches!(t, Tile::EqualsSign))
    }

    // The equation is valid if it is a number or it
    // is a proper equation with left and right hand side
    // and both expressions are parseable.
    fn verify(&self) -> Result<(), InvalidEquation> {
        let equals_positions: Vec<usize> = (0..self.tiles.len())
            .filter(|&i| matches!(self.tiles[i], Tile::EqualsSign))
            .collect();
        match equals_positions.len() {
            // Equation is just a number
            0 => {
                if self.tiles.iter().all(|tile| match tile {
                    Tile::Digit(_) => true,
                    _ => false,
                }) {
                    // If all tiles are digits, check that there is no leading zero.
                    match self.tiles[0] {
                        Tile::Digit(Digit::D0) => Err(InvalidEquation::LeadingZero),
                        _ => Ok(()),
                    }
                } else {
                    // There is a math operator - this is forbidden without equals sign!
                    Err(InvalidEquation::OperatorsWithoutEqualsSign)
                }
            }
            1 => {
                let equal_position = equals_positions[0];
                let lhs = eval_expression(&self.tiles[..equal_position])?;
                let rhs = eval_expression(&self.tiles[equal_position + 1..])?;
                if lhs == rhs {
                    Ok(())
                } else {
                    Err(InvalidEquation::SidesNotEqual(lhs, rhs))
                }
            }
            _ => Err(InvalidEquation::MoreThanOneEqualsSign),
        }
    }

    fn tile_positions(&self) -> impl Iterator<Item = (usize, usize)> {
        let (r, c) = self.initial_position;
        let f: Box<dyn Fn(usize) -> (usize, usize)> = match self.direction {
            EquationDirection::Horizontal => Box::new(move |dc| (r, c + dc)),
            EquationDirection::Vertical => Box::new(move |dr| (r + dr, c)),
        };
        (0..self.tiles.len()).map(f)
    }
}

pub enum InvalidBoard {
    NoNewTilesOnBoard,
    FirstWordMustBeOnCenter,
    NotAllTilesInOneLine,
    TilesAreNotInOneEquation,
    TilesAreNotConnected,
    InvalidEquation(EquationToVerify, InvalidEquation),
}

impl fmt::Display for InvalidBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoNewTilesOnBoard => write!(
                f,
                "No new tiles on board (if you want to pass or reroll, choose appropriate option)."
            ),
            Self::FirstWordMustBeOnCenter => {
                write!(f, "The first equation must cover the central square.")
            }
            Self::NotAllTilesInOneLine => write!(f, "All tiles must be in one line."),
            Self::TilesAreNotInOneEquation => {
                write!(f, "All tiles should belong to one main equation.")
            }
            Self::TilesAreNotConnected => write!(f, "All tiles should be connected."),
            Self::InvalidEquation(eq, err) => write!(f, "{} is not valid. {}", eq, err),
        }
    }
}

pub struct Board {
    board: Vec<Vec<BoardField>>,
    pub letters: Vec<Vec<Option<TileOnBoard>>>,
}

impl Board {
    // Build a new board with standard bonus fields alignment.
    pub fn new() -> Board {
        let mut board = vec![vec![BoardField::Standard; consts::BOARD_SIZE]; consts::BOARD_SIZE];
        for &(x, y) in &consts::DOUBLE_SYMBOL_POSITIONS {
            board[x][y] = BoardField::DoubleSymbol;
        }
        for &(x, y) in &consts::TRIPLE_SYMBOL_POSITIONS {
            board[x][y] = BoardField::TripleSymbol;
        }
        for &(x, y) in &consts::DOUBLE_EQUATION_POSITIONS {
            board[x][y] = BoardField::DoubleEquation;
        }
        for &(x, y) in &consts::TRIPLE_EQUATION_POSITIONS {
            board[x][y] = BoardField::TripleEquation;
        }
        let letters = vec![vec![None; consts::BOARD_SIZE]; consts::BOARD_SIZE];
        Board { board, letters }
    }

    // Given a position on board and a direction, build an equation containing
    // tile at the given position. The construction could fail if the position
    // does not contain a tile or the tile does not have any neighbours in given direction.
    fn construct_equation(
        &self,
        (x, y): (usize, usize),
        direction: EquationDirection,
    ) -> Option<EquationToVerify> {
        self.letters[x][y].clone().and_then(|mid_tile| {
            let (mut tiles, initial_position): (Vec<Tile>, (usize, usize)) = match direction {
                EquationDirection::Horizontal => {
                    let mut min_col = y;
                    while min_col > 0 && self.letters[x][min_col - 1].is_some() {
                        min_col -= 1;
                    }
                    (
                        (min_col..y)
                            .map(|col| self.letters[x][col].as_ref().unwrap().content())
                            .collect(),
                        (x, min_col),
                    )
                }
                EquationDirection::Vertical => {
                    let mut min_row = x;
                    while min_row > 0 && self.letters[min_row - 1][y].is_some() {
                        min_row -= 1;
                    }
                    (
                        (min_row..x)
                            .map(|row| self.letters[row][y].as_ref().unwrap().content())
                            .collect(),
                        (min_row, y),
                    )
                }
            };
            let mut tiles_after: Vec<Tile> = match direction {
                EquationDirection::Horizontal => {
                    let mut max_col = y;
                    while max_col < consts::BOARD_SIZE - 1 && self.letters[x][max_col + 1].is_some()
                    {
                        max_col += 1;
                    }
                    (y + 1..=max_col)
                        .map(|col| self.letters[x][col].as_ref().unwrap().content())
                        .collect()
                }
                EquationDirection::Vertical => {
                    let mut max_row = x;
                    while max_row < consts::BOARD_SIZE - 1 && self.letters[max_row + 1][y].is_some()
                    {
                        max_row += 1;
                    }
                    (x + 1..=max_row)
                        .map(|row| self.letters[row][y].as_ref().unwrap().content())
                        .collect()
                }
            };
            if tiles.is_empty() && tiles_after.is_empty() {
                None
            } else {
                tiles.push(mid_tile.content());
                tiles.append(&mut tiles_after);
                Some(EquationToVerify {
                    initial_position,
                    direction,
                    tiles,
                })
            }
        })
    }

    // Given a board with some temporary tiles, verify whether the new equation
    // is properly put on the board. If no, return Err with explanation.
    // If yes, the function returns score of the move and marks all temporary tiles as permenent.
    pub fn verify_and_grade_move(&mut self) -> Result<u64, InvalidBoard> {
        // The first move must cover the central square.
        if self.letters[consts::BOARD_SIZE / 2][consts::BOARD_SIZE / 2].is_none() {
            return Err(InvalidBoard::FirstWordMustBeOnCenter);
        }

        // Valid move places at least one tile on the board.
        let new_tiles_positions: Vec<(usize, usize)> = consts::board_field_positions()
            .filter(|&(x, y)| match &self.letters[x][y] {
                None => false,
                Some(tile) => match tile.status() {
                    TileStatus::Permanent => false,
                    TileStatus::Temporary => true,
                },
            })
            .collect();
        if new_tiles_positions.is_empty() {
            return Err(InvalidBoard::NoNewTilesOnBoard);
        }

        // Valid move places all tiles in one line.
        // Once we detected that all tiles are in one line, we have to check
        // that those tiles are connected.
        let new_tiles_line: BoardLine = {
            let the_row = new_tiles_positions[0].0;
            let the_col = new_tiles_positions[0].1;
            if new_tiles_positions.iter().all(|coords| coords.0 == the_row) {
                // All tiles are in the_row.
                let min_col = new_tiles_positions
                    .iter()
                    .map(|coords| coords.1)
                    .min()
                    .unwrap();
                let max_col = new_tiles_positions
                    .iter()
                    .map(|coords| coords.1)
                    .max()
                    .unwrap();
                if (min_col..max_col)
                    .map(|col| self.letters[the_row][col].as_ref())
                    .all(|letter| letter.is_some())
                {
                    BoardLine::Row(the_row)
                } else {
                    return Err(InvalidBoard::TilesAreNotInOneEquation);
                }
            } else if new_tiles_positions.iter().all(|coords| coords.1 == the_col) {
                // All tiles are in the_col.
                let min_row = new_tiles_positions
                    .iter()
                    .map(|coords| coords.0)
                    .min()
                    .unwrap();
                let max_row = new_tiles_positions
                    .iter()
                    .map(|coords| coords.0)
                    .max()
                    .unwrap();
                if (min_row..max_row)
                    .map(|row| self.letters[row][the_col].as_ref())
                    .all(|letter| letter.is_some())
                {
                    BoardLine::Column(the_col)
                } else {
                    return Err(InvalidBoard::TilesAreNotInOneEquation);
                }
            } else {
                return Err(InvalidBoard::NotAllTilesInOneLine);
            }
        };

        // Newly put tiles must be connected to tiles already on board.
        {
            // DFS starting from the central square.
            let mut dfs_stack = Vec::new();
            dfs_stack.push((consts::BOARD_SIZE / 2, consts::BOARD_SIZE / 2));
            let mut visited = vec![vec![false; consts::BOARD_SIZE]; consts::BOARD_SIZE];
            while let Some((x, y)) = dfs_stack.pop() {
                visited[x][y] = true;
                for (xx, yy) in neighbours(x, y) {
                    if self.letters[x][y].is_some() && !visited[xx][yy] {
                        dfs_stack.push((xx, yy));
                    }
                }
            }
            // Did the DFS reach all tiles?
            if consts::board_field_positions()
                .any(|(x, y)| !visited[x][y] && self.letters[x][y].is_some())
            {
                return Err(InvalidBoard::TilesAreNotConnected);
            }
        };

        // Collect all equations created by new tiles.
        // The main equation must contain all newly added tiles
        // (except for edge ase when there is only one new tile,
        // but this case is handled just fine).
        let mut equations = Vec::new();
        {
            let main_eq_direction = match new_tiles_line {
                BoardLine::Row(_) => EquationDirection::Horizontal,
                BoardLine::Column(_) => EquationDirection::Vertical,
            };
            // The main equation
            if let Some(eq) = self.construct_equation(new_tiles_positions[0], main_eq_direction) {
                equations.push(eq);
            }
            // Additional equations perpendicular to the main one
            for &pos in &new_tiles_positions {
                if let Some(eq) = self.construct_equation(pos, -main_eq_direction) {
                    equations.push(eq);
                }
            }
        }

        // Verify all those equations and grade them if they are correct.
        let mut total_grade = 0;
        for eq in equations {
            if let Err(err) = eq.verify() {
                return Err(InvalidBoard::InvalidEquation(eq, err));
            }
            if !eq.has_equals_sign() {
                continue; // Numbers without equals sign always get 0 points.
            }
            let mut multiplier = 1;
            let mut base_score = 0;
            for (x, y) in eq.tile_positions() {
                // Unwrap is safe here, because construction of equation
                // guarantees that the tile exists.
                let tile = self.letters[x][y].as_ref().unwrap();
                match tile.status() {
                    TileStatus::Permanent => base_score += tile.content().score(),
                    TileStatus::Temporary => {
                        let square = self.board[x][y];
                        base_score += tile.content().score() * square.symbol_multiplier();
                        multiplier *= square.equation_multiplier();
                    }
                }
            }
            total_grade += multiplier * base_score;
        }

        // Did the player use all his tiles (equals sign does not count)?
        // If yes, give him extra points.
        if new_tiles_positions
            .iter()
            .filter(
                |&&(x, y)| match self.letters[x][y].as_ref().unwrap().content() {
                    Tile::EqualsSign => false,
                    _ => true,
                },
            )
            .count()
            == consts::PLAYER_HAND_SIZE_WITHOUT_EQUALS_SIGN
        {
            total_grade += consts::BONUS_FOR_USING_ALL_TILES;
        }

        // Mark all tiles as permanent.
        for (x, y) in new_tiles_positions {
            if let Some(tile) = self.letters[x][y].as_mut() {
                tile.mark_as_permanent();
            }
        }

        Ok(total_grade)
    }

    // This function does not care about "schedule" actions in drawing context
    // (i. e. clearing screen, yielding timer...)
    // It assumes that the background of active game is drawn already
    // and draws the board and tiles on the board.
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let pos_to_coords = |(x, y): (usize, usize)| {
            DrawParam::default().dest([
                consts::ACTIVE_GAME_BOARD_LEFT_CORNER.0 + consts::TILE_SIZE * x as f32,
                consts::ACTIVE_GAME_BOARD_LEFT_CORNER.1 + consts::TILE_SIZE * y as f32,
            ])
        };

        // Part 1: draw board fields

        let mut standard_batch =
            SpriteBatch::new(assets.board_field_images[&BoardField::Standard].clone());
        let mut standard_positions: HashSet<(usize, usize)> =
            consts::board_field_positions().collect();

        let mut double_symbol_batch =
            SpriteBatch::new(assets.board_field_images[&BoardField::DoubleSymbol].clone());
        for pos in &consts::DOUBLE_SYMBOL_POSITIONS {
            standard_positions.remove(pos);
            double_symbol_batch.add(pos_to_coords(*pos));
        }
        double_symbol_batch.draw(ctx, DrawParam::default())?;

        let mut triple_symbol_batch =
            SpriteBatch::new(assets.board_field_images[&BoardField::TripleSymbol].clone());
        for pos in &consts::TRIPLE_SYMBOL_POSITIONS {
            standard_positions.remove(pos);
            triple_symbol_batch.add(pos_to_coords(*pos));
        }
        triple_symbol_batch.draw(ctx, DrawParam::default())?;

        let mut double_equation_batch =
            SpriteBatch::new(assets.board_field_images[&BoardField::DoubleEquation].clone());
        for pos in &consts::DOUBLE_EQUATION_POSITIONS {
            standard_positions.remove(pos);
            double_equation_batch.add(pos_to_coords(*pos));
        }
        double_equation_batch.draw(ctx, DrawParam::default())?;

        let mut triple_equation_batch =
            SpriteBatch::new(assets.board_field_images[&BoardField::TripleEquation].clone());
        for pos in &consts::TRIPLE_EQUATION_POSITIONS {
            standard_positions.remove(pos);
            triple_equation_batch.add(pos_to_coords(*pos));
        }
        triple_equation_batch.draw(ctx, DrawParam::default())?;

        for pos in standard_positions {
            standard_batch.add(pos_to_coords(pos));
        }
        standard_batch.draw(ctx, DrawParam::default())?;

        // Part 2: draw tiles on board
        // Here mixed approach is used: batching is used to draw borders (permanent/temporary tile)
        // and content of the tiles is drawn one by one.

        let mut permanent_batch =
            SpriteBatch::new(assets.tile_status_images[&TileStatus::Permanent].clone());
        let mut temporary_batch =
            SpriteBatch::new(assets.tile_status_images[&TileStatus::Temporary].clone());

        for (x, y) in consts::board_field_positions() {
            if let Some(status) = self.letters[x][y].as_ref().map(|tile| tile.status()) {
                match status {
                    TileStatus::Permanent => permanent_batch.add(pos_to_coords((x, y))),
                    TileStatus::Temporary => temporary_batch.add(pos_to_coords((x, y))),
                };
            }
        }

        permanent_batch.draw(ctx, DrawParam::default())?;
        temporary_batch.draw(ctx, DrawParam::default())?;

        let pos_to_coords_with_margin = |pos| {
            let mut coords = pos_to_coords(pos);
            coords.dest.x += consts::TILE_MARGIN;
            coords.dest.y += consts::TILE_MARGIN;
            coords
        };

        for (x, y) in consts::board_field_positions() {
            if let Some(content) = self.letters[x][y].as_ref().map(|tile| tile.content()) {
                assets.tile_images[&content].draw(ctx, pos_to_coords_with_margin((x, y)))?;
            }
        }

        Ok(())
    }
}
