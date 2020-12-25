use itertools::iproduct;

pub const BOARD_SIZE: usize = 15;
pub fn board_field_positions() -> impl Iterator<Item = (usize, usize)> {
    iproduct!(0..BOARD_SIZE, 0..BOARD_SIZE)
}

pub const MAX_NAME_LEN: usize = 16;
pub const HIGHSCORES_STORED: usize = 10;
pub const HIGHSCORES_FILE: &str = "/highscores";

pub const DIGITS_IN_BAG: usize = 6;
pub const OPERATORS_IN_BAG: usize = 9;

// Each player has 9 digits/operations and 1 equals sign,
// so 10 tiles in total.
pub const PLAYER_HAND_SIZE_WITHOUT_EQUALS_SIGN: usize = 9;
pub const PLAYER_HAND_SIZE_WITH_EQUALS_SIGN: usize = PLAYER_HAND_SIZE_WITHOUT_EQUALS_SIGN + 1;
pub const BONUS_FOR_USING_ALL_TILES: u64 = 50;

pub const DOUBLE_SYMBOL_POSITIONS: [(usize, usize); 24] = [
    (0, 3),
    (3, 0),
    (11, 0),
    (14, 3),
    (14, 11),
    (11, 14),
    (3, 14),
    (0, 11),
    (2, 6),
    (3, 7),
    (2, 8),
    (6, 2),
    (7, 3),
    (8, 2),
    (12, 6),
    (11, 7),
    (12, 8),
    (6, 12),
    (7, 11),
    (8, 12),
    (6, 6),
    (8, 6),
    (6, 8),
    (8, 8),
];
pub const TRIPLE_SYMBOL_POSITIONS: [(usize, usize); 12] = [
    (1, 5),
    (5, 5),
    (5, 1),
    (9, 1),
    (9, 5),
    (13, 5),
    (1, 9),
    (5, 9),
    (5, 13),
    (9, 13),
    (9, 9),
    (13, 9),
];
pub const DOUBLE_EQUATION_POSITIONS: [(usize, usize); 17] = [
    (1, 1),
    (2, 2),
    (3, 3),
    (4, 4),
    (10, 10),
    (11, 11),
    (12, 12),
    (13, 13),
    (1, 13),
    (2, 12),
    (3, 11),
    (4, 10),
    (10, 4),
    (11, 3),
    (12, 2),
    (13, 1),
    (7, 7),
];
pub const TRIPLE_EQUATION_POSITIONS: [(usize, usize); 8] = [
    (0, 0),
    (7, 0),
    (14, 0),
    (14, 7),
    (14, 14),
    (7, 14),
    (0, 14),
    (0, 7),
];

// Graphics - general

pub const WINDOW_SIZE: (f32, f32) = (800.0, 640.0);
pub const SMALL_FONT_SIZE: f32 = 15.0;
pub const FONT_SIZE: f32 = 24.0;
pub const LARGE_FONT_SIZE: f32 = 32.0;
pub const HUGE_FONT_SIZE: f32 = 64.0;

const X1: f32 = 15.0;
const X2: f32 = 521.0; // must be > 15.0 + BOARD_SIZE as f32 * TILE_SIZE
const X3: f32 = WINDOW_SIZE.0 - X1;

pub const TILE_SIZE: f32 = 32.0;
pub const TILE_MARGIN: f32 = 2.0;

// Lobby

pub const LOBBY_ROW_HEIGHT: f32 = LARGE_FONT_SIZE;
pub const LOBBY_FIRST_PLAYER_LEFT_CORNER: (f32, f32) = (X1 + 75.0, X1 + 75.0);
pub const LOBBY_MARGIN: f32 = 4.0;
pub const LOBBY_ROW_HEIGHT_WITH_MARGIN: f32 = LOBBY_ROW_HEIGHT + 2.0 * LOBBY_MARGIN;
pub const LOBBY_PLAYER_WIDTH: f32 = 450.0;
pub const LOBBY_ADD_REMOVE_BUTTON_X: f32 = WINDOW_SIZE.0 - X1 - LOBBY_ROW_HEIGHT_WITH_MARGIN - 75.0;
pub const LOBBY_PLAY_BUTTON_LEFT_CORNER: (f32, f32) = (310.0, 460.0);
pub const LOBBY_PLAY_BUTTON_WIDTH: f32 = WINDOW_SIZE.0 - 2.0 * LOBBY_PLAY_BUTTON_LEFT_CORNER.0;

// Board

pub const ACTIVE_GAME_BOARD_LEFT_CORNER: (f32, f32) = (X1, X1);

pub fn pos_to_board_coords(x: f32, y: f32) -> Option<(usize, usize)> {
    if x < ACTIVE_GAME_BOARD_LEFT_CORNER.0 || y < ACTIVE_GAME_BOARD_LEFT_CORNER.1 {
        return None;
    }
    let ux = ((x - ACTIVE_GAME_BOARD_LEFT_CORNER.0) / TILE_SIZE) as usize;
    let uy = ((y - ACTIVE_GAME_BOARD_LEFT_CORNER.1) / TILE_SIZE) as usize;
    if ux < BOARD_SIZE && uy < BOARD_SIZE {
        Some((ux, uy))
    } else {
        None
    }
}

// Hand

pub const ACTIVE_GAME_HAND_LEFT_CORNER: (f32, f32) = (X1, X2);
pub const ACTIVE_GAME_HAND_MARGIN: f32 = 4.0;

pub fn pos_to_hand_coords(tiles_in_hand: usize, x: f32, y: f32) -> Option<usize> {
    if x < ACTIVE_GAME_HAND_LEFT_CORNER.0 + ACTIVE_GAME_HAND_MARGIN
        || y < ACTIVE_GAME_HAND_LEFT_CORNER.1 + ACTIVE_GAME_HAND_MARGIN
        || y > ACTIVE_GAME_HAND_LEFT_CORNER.1 + ACTIVE_GAME_HAND_MARGIN + TILE_SIZE
    {
        return None;
    }
    let ux = ((x - ACTIVE_GAME_HAND_LEFT_CORNER.0) / TILE_SIZE) as usize;
    if ux < tiles_in_hand {
        Some(ux)
    } else {
        None
    }
}

// Players' panel

pub const ACTIVE_GAME_FIRST_PLAYER_LEFT_CORNER: (f32, f32) = (X2, X1);
pub const ACTIVE_GAME_PLAYER_MARGIN: f32 = 4.0;
pub const ACTIVE_GAME_PLAYER_SIZE: (f32, f32) =
    (X3 - X2, FONT_SIZE + 2.0 * ACTIVE_GAME_PLAYER_MARGIN);
pub const ACTIVE_GAME_PLAYER_NAME_LEN: f32 = 185.0;

// Control buttons

pub const ACTIVE_GAME_BUTTON_MARGIN: f32 = ACTIVE_GAME_PLAYER_MARGIN;
pub const ACTIVE_GAME_BUTTON_SIZE: (f32, f32) =
    (X3 - X2, FONT_SIZE + 2.0 * ACTIVE_GAME_BUTTON_MARGIN);
pub const ACTIVE_GAME_BUTTON_FREE_SPACE: f32 = FONT_SIZE / 2.0;

// Popup

pub const ACTIVE_GAME_POPUP_Y_MARGIN: f32 = 50.0;
pub const ACTIVE_GAME_POPUP_MARGIN: f32 = ACTIVE_GAME_PLAYER_MARGIN;

// Summary

pub const HIGHSCORES_HEADLINE: &str = "New highscores!";
pub const SUMMARY_HEADLINE: &str = "The final results are:";
pub const SUMMARY_HEADLINE_LEFT_CORNER: (f32, f32) = (X1, X1);
pub const SUMMARY_TEXT_SIZE: (f32, f32) = (X3 - X1, FONT_SIZE);
pub const SUMMARY_TEXT_ROW_SIZE: f32 = 1.7 * FONT_SIZE;
pub const SUMMARY_TEXT_FOOTER_Y: f32 = WINDOW_SIZE.1 - 2.0 * SMALL_FONT_SIZE;
