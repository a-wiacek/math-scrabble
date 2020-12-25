use crate::assets::Assets;
use crate::board::{Board, InvalidBoard};
use crate::consts::*;
use crate::tiles::{Tile, TileOnBoard, TileStatus};
use ggez::{
    graphics::{self, spritebatch::SpriteBatch, Drawable},
    input::mouse::{self, MouseButton},
    Context, GameResult,
};
use graphics::DrawParam;
use log::{error, info};
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::fmt;

pub struct Player {
    name: String,
    hand: Vec<Tile>, // this includes equals sign
    score: u64,
}

impl Player {
    pub fn hand_len(&self) -> usize {
        self.hand.len()
    }
}

#[derive(Eq, PartialEq, Clone)]
enum ControlButton {
    ConfirmMove,
    RerollTtiles,
    PassTurn,
    ConfirmReroll,
    CancelReroll,
}

impl fmt::Display for ControlButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControlButton::ConfirmMove => f.write_str("Confirm move"),
            ControlButton::RerollTtiles => f.write_str("Reroll tiles"),
            ControlButton::PassTurn => f.write_str("Pass turn"),
            ControlButton::ConfirmReroll => f.write_str("Confirm reroll"),
            ControlButton::CancelReroll => f.write_str("Cancel reroll"),
        }
    }
}

#[derive(Clone)]
pub struct PlacingTiles {
    tile_in_move: Option<Tile>,
}

impl PlacingTiles {
    fn new() -> PlacingTiles {
        PlacingTiles { tile_in_move: None }
    }
}

#[derive(Clone)]
pub struct Rerolling {
    // tiles_selected[i] = is i-th tile rerolled?
    // Tile with equals sign is never selected
    tiles_selected: Vec<bool>,
}

impl Rerolling {
    fn new() -> Rerolling {
        Rerolling {
            tiles_selected: vec![false; PLAYER_HAND_SIZE_WITH_EQUALS_SIGN],
        }
    }
}

#[derive(Clone)]
pub enum PlayerAction {
    PlacingTiles(PlacingTiles),
    Rerolling(Rerolling),
}

enum ActiveGamePopup {
    // First &str refers to the player that has finished his move.
    // Next &str refers to the next player.
    // Display is altered when only one player is in game
    // (when references are equal).
    Scored(u64, bool),
    Passed(usize),
    Rerolled(usize, bool),
    NotEnoughTilesToReroll,
    NoTilesSelectedToReroll,
    InvalidBoard(InvalidBoard),
}

impl fmt::Display for ActiveGamePopup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActiveGamePopup::Scored(points, many_players) => {
                if *many_players {
                    write!(
                        f,
                        "You have scored {} points. Next player's turn now.",
                        points
                    )
                } else {
                    write!(f, "You have scored {} points.", points)
                }
            }
            ActiveGamePopup::Passed(rerolls) => {
                // In case of one-player game, this popup will not appear.
                let nth = match rerolls {
                    1 => "1st",
                    2 => "2nd",
                    3 => "3rd",
                    _ => "4th",
                };
                write!(
                    f,
                    "You have passed your turn ({} reroll in a row). Next player's turn now.",
                    nth
                )
            }
            ActiveGamePopup::Rerolled(tiles, many_players) => {
                if *many_players {
                    write!(
                        f,
                        "You have rerolled {} tile{}. Next player's turn now.",
                        tiles,
                        if tiles > &1 { "s" } else { "" },
                    )
                } else {
                    write!(f, "You have rerolled {} tiles.", tiles)
                }
            }
            ActiveGamePopup::NotEnoughTilesToReroll => {
                write!(f, "Not enough tiles in bag to reroll.")
            }
            ActiveGamePopup::NoTilesSelectedToReroll => {
                write!(f, "Select at least one tile to reroll.")
            }
            ActiveGamePopup::InvalidBoard(reason) => write!(f, "Invalid move. {}", reason),
        }
    }
}

pub struct ActiveGame {
    players: Vec<Player>,
    player_index: usize,
    player_action: PlayerAction,
    passes_in_row: usize,
    tiles_in_bag: Vec<Tile>,
    board: Board,
    board_popup: Option<ActiveGamePopup>,
    rng: ThreadRng,
}

// constructor + simple getters / setters
impl ActiveGame {
    pub fn new(player_names: Vec<String>) -> ActiveGame {
        let mut tiles_in_bag = Tile::initial_bag_of_tiles();
        let mut rng = rand::thread_rng();
        tiles_in_bag.shuffle(&mut rng);
        ActiveGame {
            players: player_names
                .into_iter()
                .map(|name| {
                    let at = tiles_in_bag.len() - PLAYER_HAND_SIZE_WITHOUT_EQUALS_SIGN;
                    let mut hand = tiles_in_bag.split_off(at);
                    hand.push(Tile::EqualsSign);
                    info!(
                        "Initial tiles of player {}: {}",
                        name,
                        hand.iter().map(|tile| tile.to_string()).collect::<String>()
                    );
                    Player {
                        name,
                        hand,
                        score: 0,
                    }
                })
                .collect(),
            player_index: 0,
            player_action: PlayerAction::PlacingTiles(PlacingTiles::new()),
            passes_in_row: 0,
            tiles_in_bag,
            board: Board::new(),
            board_popup: None,
            rng,
        }
    }

    fn current_player(&self) -> &Player {
        self.players.get(self.player_index).unwrap()
    }

    fn current_player_mut(&mut self) -> &mut Player {
        self.players.get_mut(self.player_index).unwrap()
    }

    fn buttons_min_y(&self) -> f32 {
        ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER.1
            + ACTIVE_GAME_PLAYER_SIZE.1 * self.players.len() as f32
            + 2.0 * ACTIVE_GAME_BUTTON_FREE_SPACE
    }

    fn pos_to_button(&self, x: f32, y: f32) -> Option<ControlButton> {
        if x < ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER.0
            || x > ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER.0 + ACTIVE_GAME_PLAYER_SIZE.0
            || y < self.buttons_min_y()
        {
            return None;
        }
        let opt = {
            let num = y - self.buttons_min_y();
            let den = ACTIVE_GAME_BUTTON_SIZE.1 + ACTIVE_GAME_BUTTON_FREE_SPACE;
            let rem = num % den;
            if rem > ACTIVE_GAME_BUTTON_SIZE.1 {
                None
            } else {
                Some((num / den) as usize)
            }
        };
        match self.player_action {
            PlayerAction::PlacingTiles(_) => match opt {
                Some(0) => Some(ControlButton::ConfirmMove),
                Some(1) => Some(ControlButton::RerollTtiles),
                Some(2) => Some(ControlButton::PassTurn),
                _ => None,
            },
            PlayerAction::Rerolling(_) => match opt {
                Some(0) => Some(ControlButton::ConfirmReroll),
                Some(1) => Some(ControlButton::CancelReroll),
                _ => None,
            },
        }
    }

    fn switching_players(&self) -> bool {
        use ActiveGamePopup::*;
        self.players.len() > 1
            && matches!(self.board_popup, Some(Passed(_)) | Some(Rerolled(_, _)) | Some(Scored(_, _)))
    }

    // This function is called to check if the game is finished.
    // The vector contains unsorted results of all players.
    pub fn check_finish(&self) -> Option<Vec<(String, u64)>> {
        if self.passes_in_row >= self.players.len() {
            Some(
                self.players
                    .iter()
                    .map(|player| (player.name.clone(), player.score))
                    .collect(),
            )
        } else {
            None
        }
    }
}

// core game logic functions + draw
impl ActiveGame {
    // Move a tile from hand. The function assumes that x is valid.
    pub fn pick_up_tile_from_hand(&mut self, x: usize) -> bool {
        info!("Trying to pick up tile from hand at position {}.", x);
        let tile = self.current_player().hand[x].clone();
        // Is player in "placing tiles" mode?
        if let PlayerAction::PlacingTiles(action) = &mut self.player_action {
            // Is any tile in move?
            if action.tile_in_move.is_none() {
                info!(
                    "Succesfully picked up tile {} from hand at position {}.",
                    tile, x
                );
                action.tile_in_move = Some(tile);
                self.current_player_mut().hand.remove(x);
                return true;
            } else {
                error!(
                    "Tried to pick up tile while there is already another tile {} in hand.",
                    action.tile_in_move.as_ref().unwrap()
                );
            }
        } else {
            error!("Tried to pick up tile while player action is rerolling.");
        }
        false
    }

    // Move a temporary tile from board.
    // The function assumes that x and y are valid (0 <= x, y < BOARD_SIZE).
    pub fn pick_up_tile_from_board(&mut self, x: usize, y: usize) -> bool {
        info!(
            "Trying to pick up tile from board at position ({}, {}).",
            x, y
        );
        // Is player in "placing tiles" mode?
        if let PlayerAction::PlacingTiles(action) = &mut self.player_action {
            // Is any tile in move?
            if action.tile_in_move.is_none() {
                // Does the square contain a tile?
                if let Some(tile) = self.board.letters[x][y].clone() {
                    // Is the tile temporary?
                    if tile.status() == TileStatus::Temporary {
                        info!("Successfully picked up {}.", tile.content());
                        self.board.letters[x][y] = None;
                        action.tile_in_move = Some(tile.content());
                        return true;
                    } else {
                        info!("Did not pick up tile: it has permanent status.");
                    }
                } else {
                    info!("The board square does not contain any tile.")
                }
            } else {
                error!(
                    "Tried to pick up tile while there is already another tile {} in hand.",
                    action.tile_in_move.as_ref().unwrap()
                );
            }
        } else {
            error!("Tried to pick up tile while player action is rerolling.");
        }
        false
    }

    // Drop the tile in move back to hand.
    pub fn put_tile_to_hand(&mut self) {
        if let PlayerAction::PlacingTiles(action) = &mut self.player_action {
            // Is any tile in move?
            if let Some(tile) = action.tile_in_move.clone() {
                info!("Tile {} goes back from move to hand", tile);
                action.tile_in_move = None;
                self.current_player_mut().hand.push(tile);
            } else {
                error!("Tried to put tile back to hand, but there is no tile in move.");
            }
        } else {
            error!("Tried to put tile back to hand while player action is rerolling.");
        }
    }

    // Drop the tile in move on board. Boolean value returns information whether the tile was moved.
    // The function assumes that x and y are valid (0 <= x, y < BOARD_SIZE).
    pub fn put_tile_on_board(&mut self, x: usize, y: usize) -> bool {
        info!("Trying to put tile in hand on ({}, {}).", x, y);
        // Is player in "placing tiles" mode?
        if let PlayerAction::PlacingTiles(action) = &mut self.player_action {
            // Is any tile in move?
            if let Some(tile) = action.tile_in_move.clone() {
                // Is the selected field empty?
                if self.board.letters[x][y].is_none() {
                    info!("Successfully put the tile in hand.");
                    action.tile_in_move = None;
                    self.board.letters[x][y] = Some(TileOnBoard::new(tile));
                    return true;
                } else {
                    info!(
                        "Selected square already contains another tile {}",
                        self.board.letters[x][y].as_ref().unwrap().content()
                    );
                }
            } else {
                error!("Tried to put tile on board, but there is no tile in move.");
            }
        } else {
            error!("Tried to put tile on board while player action is rerolling.");
        }
        false
    }

    // If the board field has temporary tile, put it back in
    // the current player's hand. Boolean value returns information
    // whether the tile was moved.
    // The function assumes that x and y are valid (0 <= x, y < BOARD_SIZE).
    pub fn pop_tile_from_board(&mut self, x: usize, y: usize) -> bool {
        info!("Trying to pop tile from board at ({}, {}).", x, y);
        // Does the selected field contain a tile?
        if let Some(tile_on_board) = self.board.letters[x][y].clone() {
            // Is the tile temporary?
            if tile_on_board.status() == TileStatus::Temporary {
                info!("Successfully popped the tile from board.");
                self.current_player_mut().hand.push(tile_on_board.content());
                self.board.letters[x][y] = None;
                return true;
            } else {
                info!("This square has permanent tile.");
            }
        } else {
            info!("This square does not have any tile.");
        }
        false
    }

    // Move all temporary tiles from the board to the current player's hand.
    pub fn pop_all_tiles_from_board(&mut self) {
        info!("Popping all tiles on board.");
        for (x, y) in board_field_positions() {
            if let Some(tile_on_board) = self.board.letters[x][y].clone() {
                if tile_on_board.status() == TileStatus::Temporary {
                    info!("Popping at ({}, {}).", x, y);
                    self.current_player_mut().hand.push(tile_on_board.content());
                    self.board.letters[x][y] = None;
                }
            }
        }
        info!("Finished popping all tiles on board.");
    }

    // This function is used when player decides which tiles does he want to reroll.
    // It is assumed that 0 <= pos < PLAYER_HAND_SIZE_WITH_EQUALS_SIGN,
    // since during "reroll phase" all tiles must be in hand.
    pub fn select_tile_in_hand(&mut self, pos: usize) {
        info!("Trying to select tile in hand at position {}.", pos);
        let equals_index = self
            .current_player()
            .hand
            .iter()
            .position(|el| matches!(el, Tile::EqualsSign))
            .unwrap(); // Should never fail, since player should always have exactly one equals sign
        if pos != equals_index {
            if let PlayerAction::Rerolling(reroll) = &mut self.player_action {
                reroll.tiles_selected[pos] = !reroll.tiles_selected[pos];
                if reroll.tiles_selected[pos] {
                    info!("Successfully selected tile.");
                } else {
                    info!("Successfully unselected tile.");
                }
            }
        } else {
            info!("Not selecting tile, since equals sign is at that position.")
        }
    }

    fn refill_current_player_hand(&mut self) {
        info!("Refilling current player's hand.");
        let hand = &mut self.current_player_mut().hand;
        if !hand.contains(&Tile::EqualsSign) {
            hand.push(Tile::EqualsSign);
        }
        let missing = PLAYER_HAND_SIZE_WITH_EQUALS_SIGN - hand.len();
        for _ in 0..missing {
            if let Some(tile) = self.tiles_in_bag.pop() {
                info!("New tile: {}.", tile);
                self.current_player_mut().hand.push(tile);
            } else {
                info!("No more tiles in bag.");
                break;
            }
        }
        info!("Finished refilling current player's hand.");
    }

    // Select the next player and clean up.
    fn next_player(&mut self) {
        info!("Starting next player's move.");
        self.player_index += 1;
        if self.player_index == self.players.len() {
            self.player_index = 0;
        }
        self.player_action = PlayerAction::PlacingTiles(PlacingTiles::new());
    }

    // Once the player decides that he made his move, this function is called.
    // If the move is correct, the score of the current player is updated and
    // the next player is selected. If the move is incorrect, board_error
    // field is set up and the board remains unchanged.
    pub fn confirm_move(&mut self) {
        info!("Validating current player's move.");
        match self.board.verify_and_grade_move() {
            Ok(score) => {
                info!("Move scored {} points.", score);
                self.current_player_mut().score += score;
                self.refill_current_player_hand();
                self.next_player();
                self.board_popup = Some(ActiveGamePopup::Scored(score, self.players.len() > 1));
                self.passes_in_row = 0;
            }
            Err(err) => {
                info!("Invalid move: {}", err);
                self.board_popup = Some(ActiveGamePopup::InvalidBoard(err))
            }
        }
    }

    // The current player decides to pass his turn.
    pub fn pass_turn(&mut self) {
        info!("Current player passes his turn.");
        self.pop_all_tiles_from_board();
        self.next_player();
        self.passes_in_row += 1;
        if self.players.len() > 1 {
            self.board_popup = Some(ActiveGamePopup::Passed(self.passes_in_row));
        }
    }

    // The current player decides to reroll some of his tiles.
    // This turns on "reroll mode". All temporary tiles are popped from the board.
    // While reroll mode is active, the player can only select tiles in is hand
    // (excluding equals sign). Once the player selected a non-zero number of tiles,
    // he can confirm his decision to reroll.
    // Of course, the player can always cancel reroll and go back to placing tiles.
    // The player can reroll only if at least
    // PLAYER_HAND_SIZE_WITHOUT_EQUALS_SIGN tiles are in the bag.
    // Function may fail. If it does, board popup is set.
    pub fn init_reroll(&mut self) {
        info!("Trying to switch to reroll action.");
        if self.tiles_in_bag.len() >= PLAYER_HAND_SIZE_WITHOUT_EQUALS_SIGN {
            self.pop_all_tiles_from_board();
            self.player_action = PlayerAction::Rerolling(Rerolling::new());
            info!("Successfully switched to reroll action.");
        } else {
            self.board_popup = Some(ActiveGamePopup::NotEnoughTilesToReroll);
            info!("Failed to switch to reroll action: not enough tiles in bag.");
        }
    }

    pub fn cancel_reroll(&mut self) {
        info!("Switchin back to placing tiles action.");
        self.player_action = PlayerAction::PlacingTiles(PlacingTiles::new());
    }

    // The function fails if no tiles were selected.
    // If it fails, appropriate board popup is selected.
    // If it succeeds, control is handed over to the next player.
    // Note that reroll is different from pass and resets passes counter.
    pub fn confirm_reroll(&mut self) {
        info!("Confirming reroll.");
        if let PlayerAction::Rerolling(reroll) = &self.player_action {
            let to_be_rerolled: Vec<usize> = (0..PLAYER_HAND_SIZE_WITH_EQUALS_SIGN)
                // Rev is important later on: since we want to extract tiles at those positions,
                // we want to start from the biggest indexes.
                .rev()
                .filter(|&i| reroll.tiles_selected[i])
                .collect();
            let l = to_be_rerolled.len();
            if l > 0 {
                info!("Rerolling {} tiles.", l);
                let mut discarded_tiles = Vec::<Tile>::with_capacity(l);
                let hand = &mut self.current_player_mut().hand;
                for pos in to_be_rerolled {
                    discarded_tiles.push(hand.remove(pos));
                }
                self.refill_current_player_hand();
                self.tiles_in_bag.extend(discarded_tiles);
                self.tiles_in_bag.shuffle(&mut self.rng);
                self.next_player();
                self.board_popup = Some(ActiveGamePopup::Rerolled(l, self.players.len() > 1));
                self.passes_in_row = 0;
            } else {
                info!("Not rerolling: no tiles selected.");
                self.board_popup = Some(ActiveGamePopup::NoTilesSelectedToReroll);
            }
        } else {
            error!("Confirming reroll, but the current action is placing tiles.")
        }
    }

    fn apply_control_button(&mut self, button: ControlButton) {
        info!("Pressed control button \"{}\".", button);
        match button {
            ControlButton::ConfirmMove => self.confirm_move(),
            ControlButton::RerollTtiles => self.init_reroll(),
            ControlButton::PassTurn => self.pass_turn(),
            ControlButton::ConfirmReroll => self.confirm_reroll(),
            ControlButton::CancelReroll => self.cancel_reroll(),
        }
    }

    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult {
        // Part 1: Draw board
        self.board.draw(ctx, assets)?;

        {
            // Part 2: Draw current player's hand
            let hand = &self.current_player().hand;
            let pos_to_coords = |x: usize| {
                graphics::DrawParam::default().dest([
                    ACTIVE_GAME_HAND_LEFT_CORNER.0 + ACTIVE_GAME_HAND_MARGIN + TILE_SIZE * x as f32,
                    ACTIVE_GAME_HAND_LEFT_CORNER.1 + ACTIVE_GAME_HAND_MARGIN,
                ])
            };
            let pos_to_coords_with_margin = |pos| {
                let mut coords = pos_to_coords(pos);
                coords.dest.x += TILE_MARGIN;
                coords.dest.y += TILE_MARGIN;
                coords
            };

            // * Draw white rectangle (container for tiles in hand) and border.
            let x = ACTIVE_GAME_HAND_LEFT_CORNER.0;
            let y = ACTIVE_GAME_HAND_LEFT_CORNER.1;
            let w = TILE_SIZE * PLAYER_HAND_SIZE_WITH_EQUALS_SIGN as f32
                + 2.0 * ACTIVE_GAME_HAND_MARGIN;
            let h = TILE_SIZE + 2.0 * ACTIVE_GAME_HAND_MARGIN;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, y, w, h),
                (255, 255, 255).into(),
            )?
            .draw(ctx, graphics::DrawParam::default())?;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Stroke(
                    graphics::StrokeOptions::default().with_line_width(ACTIVE_GAME_HAND_MARGIN),
                ),
                graphics::Rect::new(x, y, w, h),
                (0, 0, 0).into(),
            )?
            .draw(ctx, graphics::DrawParam::default())?;

            // * Unless turn is passed to another player,
            if !self.switching_players() {
                // ** Draw fields depending on whether a tile is selected or not
                let not_selected_tile = assets.tile_status_images[&TileStatus::Permanent].clone();
                let selected_tile = assets.tile_status_images[&TileStatus::Temporary].clone();
                match &self.player_action {
                    PlayerAction::PlacingTiles(_) => {
                        let mut batch = SpriteBatch::new(not_selected_tile);
                        for i in 0..hand.len() {
                            batch.add(pos_to_coords(i));
                        }
                        batch.draw(ctx, graphics::DrawParam::default())?;
                    }
                    PlayerAction::Rerolling(rerolling) => {
                        let selected_batch = &mut SpriteBatch::new(selected_tile);
                        let not_selected_batch = &mut SpriteBatch::new(not_selected_tile);

                        for i in 0..hand.len() {
                            if rerolling.tiles_selected[i] {
                                selected_batch.add(pos_to_coords(i));
                            } else {
                                not_selected_batch.add(pos_to_coords(i));
                            }
                        }

                        selected_batch.draw(ctx, graphics::DrawParam::default())?;
                        not_selected_batch.draw(ctx, graphics::DrawParam::default())?;
                    }
                }

                // ** Draw content of tiles
                for (i, tile) in hand.iter().enumerate() {
                    assets.tile_images[tile].draw(ctx, pos_to_coords_with_margin(i))?;
                }
            }
        }

        // Part 3: Draw scoreboard
        let mut draw_player = |i: usize| -> GameResult {
            let x = ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER.0;
            let y = ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER.1 + ACTIVE_GAME_PLAYER_SIZE.1 * i as f32;
            let w = ACTIVE_GAME_PLAYER_SIZE.0;
            let h = ACTIVE_GAME_PLAYER_SIZE.1;

            // Rectangle in the background
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, y, w, h),
                if i == self.player_index {
                    (155, 155, 60).into() // Highlight current player
                } else {
                    (100, 100, 100).into()
                },
            )?
            .draw(ctx, graphics::DrawParam::default())?;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Stroke(
                    graphics::StrokeOptions::default().with_line_width(ACTIVE_GAME_PLAYER_MARGIN),
                ),
                graphics::Rect::new(x, y, w, h),
                (0, 0, 0).into(),
            )?
            .draw(ctx, graphics::DrawParam::default())?;

            // Player name
            let player = self.players.get(i).unwrap();

            let mut name_text = graphics::Text::new(graphics::TextFragment {
                text: player.name.clone(),
                color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
                scale: Some(graphics::Scale::uniform(FONT_SIZE)),
                ..Default::default()
            });
            name_text
                .set_bounds(
                    [
                        ACTIVE_GAME_PLAYER_NAME_LEN,
                        h - 2.0 * ACTIVE_GAME_PLAYER_MARGIN,
                    ],
                    graphics::Align::Left,
                )
                .draw(
                    ctx,
                    graphics::DrawParam::default()
                        .dest([x + ACTIVE_GAME_PLAYER_MARGIN, y + ACTIVE_GAME_PLAYER_MARGIN]),
                )?;

            // Player score
            let mut score_text = graphics::Text::new(graphics::TextFragment {
                text: player.score.to_string(),
                color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
                scale: Some(graphics::Scale::uniform(FONT_SIZE)),
                ..Default::default()
            });
            score_text
                .set_bounds(
                    [
                        w - ACTIVE_GAME_PLAYER_NAME_LEN - 2.0 * ACTIVE_GAME_PLAYER_MARGIN,
                        h - 2.0 * ACTIVE_GAME_PLAYER_MARGIN,
                    ],
                    graphics::Align::Right,
                )
                .draw(
                    ctx,
                    graphics::DrawParam::default().dest([
                        x + ACTIVE_GAME_PLAYER_NAME_LEN + ACTIVE_GAME_PLAYER_MARGIN,
                        y + ACTIVE_GAME_PLAYER_MARGIN,
                    ]),
                )?;

            Ok(())
        };
        for i in 0..self.players.len() {
            draw_player(i)?;
        }

        // Part 4: Draw buttons
        let mut draw_button = |pos: usize, button: ControlButton| -> GameResult {
            let x = ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER.0;
            let y = self.buttons_min_y()
                + pos as f32 * (ACTIVE_GAME_BUTTON_SIZE.1 + ACTIVE_GAME_BUTTON_FREE_SPACE);

            let w = ACTIVE_GAME_BUTTON_SIZE.0;
            let h = ACTIVE_GAME_BUTTON_SIZE.1;

            let button_hovered = {
                let mouse_pos = mouse::position(ctx);
                self.board_popup.is_none()
                    && match &self.player_action {
                        PlayerAction::PlacingTiles(placing) => placing.tile_in_move.is_none(),
                        PlayerAction::Rerolling(_) => true,
                    }
                    && mouse_pos.x >= x
                    && mouse_pos.y >= y
                    && mouse_pos.x <= x + w
                    && mouse_pos.y <= y + h
            };

            // Rectangle in the background
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, y, w, h),
                (205, 145, 0).into(),
            )?
            .draw(ctx, graphics::DrawParam::default())?;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Stroke(
                    graphics::StrokeOptions::default().with_line_width(ACTIVE_GAME_BUTTON_MARGIN),
                ),
                graphics::Rect::new(x, y, w, h),
                if button_hovered {
                    (100, 100, 100).into()
                } else {
                    (0, 0, 0).into()
                },
            )?
            .draw(ctx, graphics::DrawParam::default())?;

            // Text on the button
            let mut text = graphics::Text::new(graphics::TextFragment {
                text: button.to_string(),
                color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
                scale: Some(graphics::Scale::uniform(FONT_SIZE)),
                ..Default::default()
            });
            text.set_bounds(
                [
                    w - 2.0 * ACTIVE_GAME_BUTTON_MARGIN,
                    h - 2.0 * ACTIVE_GAME_BUTTON_MARGIN,
                ],
                graphics::Align::Center,
            )
            .draw(
                ctx,
                graphics::DrawParam::default()
                    .dest([x + ACTIVE_GAME_BUTTON_MARGIN, y + ACTIVE_GAME_BUTTON_MARGIN]),
            )?;
            Ok(())
        };
        match self.player_action {
            PlayerAction::PlacingTiles(_) => {
                draw_button(0, ControlButton::ConfirmMove)?;
                draw_button(1, ControlButton::RerollTtiles)?;
                draw_button(2, ControlButton::PassTurn)?;
            }
            PlayerAction::Rerolling(_) => {
                draw_button(0, ControlButton::ConfirmReroll)?;
                draw_button(1, ControlButton::CancelReroll)?;
            }
        }

        // Part 5: Draw popup
        let mut draw_popup = |popup: &ActiveGamePopup| -> GameResult {
            // Rectangle in the background
            let x = ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER.0;
            let y = self.buttons_min_y()
                + match &self.player_action {
                    PlayerAction::PlacingTiles(_) => 3.0,
                    PlayerAction::Rerolling(_) => 2.0,
                } * (ACTIVE_GAME_BUTTON_SIZE.1 + ACTIVE_GAME_BUTTON_FREE_SPACE)
                + ACTIVE_GAME_BUTTON_FREE_SPACE;
            let w = ACTIVE_GAME_PLAYER_SIZE.0;
            let h = WINDOW_SIZE.1 - y - ACTIVE_GAME_POPUP_Y_MARGIN;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, y, w, h),
                (160, 0, 0).into(),
            )?
            .draw(ctx, graphics::DrawParam::default())?;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Stroke(
                    graphics::StrokeOptions::default().with_line_width(ACTIVE_GAME_POPUP_MARGIN),
                ),
                graphics::Rect::new(x, y, w, h),
                (0, 0, 0).into(),
            )?
            .draw(ctx, graphics::DrawParam::default())?;

            // Popup text

            let mut text = graphics::Text::new(graphics::TextFragment {
                text: popup.to_string(),
                color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
                scale: Some(graphics::Scale::uniform(FONT_SIZE)),
                ..Default::default()
            });
            text.set_bounds(
                [
                    w - 2.0 * ACTIVE_GAME_POPUP_MARGIN,
                    h - 2.0 * ACTIVE_GAME_POPUP_MARGIN,
                ],
                graphics::Align::Center,
            )
            .draw(
                ctx,
                graphics::DrawParam::default()
                    .dest([x + ACTIVE_GAME_POPUP_MARGIN, y + ACTIVE_GAME_POPUP_MARGIN]),
            )?;

            // "Click anywhere to continue"

            let mut text = graphics::Text::new(graphics::TextFragment {
                text: "Click anywhere to continue".to_string(),
                color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
                scale: Some(graphics::Scale::uniform(SMALL_FONT_SIZE)),
                ..Default::default()
            });
            text.set_bounds(
                [
                    w - 2.0 * ACTIVE_GAME_POPUP_MARGIN,
                    h - 2.0 * ACTIVE_GAME_POPUP_MARGIN,
                ],
                graphics::Align::Center,
            )
            .draw(
                ctx,
                graphics::DrawParam::default().dest([
                    x + ACTIVE_GAME_POPUP_MARGIN,
                    WINDOW_SIZE.1
                        - ACTIVE_GAME_POPUP_Y_MARGIN
                        - SMALL_FONT_SIZE
                        - ACTIVE_GAME_POPUP_MARGIN,
                ]),
            )?;

            Ok(())
        };
        if let Some(popup) = &self.board_popup {
            draw_popup(popup)?;
        }

        // Part 6: Draw the tile in move
        if let PlayerAction::PlacingTiles(placing_tiles) = &self.player_action {
            if let Some(tile) = &placing_tiles.tile_in_move {
                let mouse_pos = mouse::position(ctx);
                let x1 = mouse_pos.x - TILE_SIZE / 2.0;
                let y1 = mouse_pos.y - TILE_SIZE / 2.0;
                let x2 = x1 + TILE_MARGIN;
                let y2 = y1 + TILE_MARGIN;
                assets.tile_status_images[&TileStatus::Temporary]
                    .draw(ctx, DrawParam::default().dest([x1, y1]))?;
                assets.tile_images[tile].draw(ctx, DrawParam::default().dest([x2, y2]))?;
            }
        }

        Ok(())
    }
}

// mutating state through events
impl ActiveGame {
    pub fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if self.board_popup.is_none() {
            // Possible actions:
            // * picking up a tile that the player wants to drag,
            // * selecting tile to reroll,
            // * starting to press a control button.
            // All done with left mouse button.
            if let MouseButton::Left = button {
                match self.player_action {
                    PlayerAction::PlacingTiles(_) => {
                        let tiles_in_hand = self.current_player().hand_len();
                        if let Some((x, y)) = pos_to_board_coords(x, y) {
                            self.pick_up_tile_from_board(x, y);
                        } else if let Some(x) = pos_to_hand_coords(tiles_in_hand, x, y) {
                            self.pick_up_tile_from_hand(x);
                        } else if let Some(button) = self.pos_to_button(x, y) {
                            self.apply_control_button(button);
                        }
                    }
                    PlayerAction::Rerolling(_) => {
                        if let Some(x) = pos_to_hand_coords(PLAYER_HAND_SIZE_WITH_EQUALS_SIGN, x, y)
                        {
                            self.select_tile_in_hand(x);
                        } else if let Some(button) = self.pos_to_button(x, y) {
                            self.apply_control_button(button);
                        }
                    }
                }
            }
        } else {
            // If there is a popup, the only possible action is to remove it.
            self.board_popup = None;
        }
    }

    pub fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        // Ignore all but left and right mouse buttons.
        if !matches!(button, MouseButton::Left | MouseButton::Right) {
            return;
        }
        // The only valid action for right click is to pop temporary tiles from board to hand.
        // Clicking on a board square pops square from that position (if it exists).
        // Clicking outside of board pops all temporary squares.
        if let MouseButton::Right = button {
            if let Some((x, y)) = pos_to_board_coords(x, y) {
                self.pop_tile_from_board(x, y);
            } else {
                self.pop_all_tiles_from_board();
            }
            return;
        }
        // Dropping tile from hand
        if let PlayerAction::PlacingTiles(placing) = &self.player_action {
            if placing.tile_in_move.is_some() {
                // The tile is dropped: if the mouse is over an empty board field,
                // the tile goes there. Otherwise, the tile goes back to the hand.
                if let Some((x, y)) = pos_to_board_coords(x, y) {
                    if !self.put_tile_on_board(x, y) {
                        self.put_tile_to_hand();
                    }
                } else {
                    self.put_tile_to_hand();
                }
            }
            return;
        }
    }
}
