use crate::assets::Assets;
use crate::consts::*;
use ggez::{
    event::{KeyCode, KeyMods},
    graphics::{self, Drawable},
    input::mouse::{self, MouseButton},
    Context, GameResult,
};
use log::info;

#[derive(Eq, PartialEq, Clone)]
enum ControlButton {
    AddPlayer,
    RemovePlayer(usize),
    StartGame,
}

#[derive(Eq, PartialEq, Clone)]
enum MouseAction {
    ControlButton(ControlButton),
    PlayerName(usize),
}

pub struct Lobby {
    player_names: Vec<String>, // Length of this vector must be between 1 and 4
    player_selected: usize,
    mouse_action: Option<MouseAction>,
    ready: bool,
}

impl Lobby {
    pub fn new() -> Lobby {
        Lobby {
            player_names: vec![String::from("Player 1"), String::from("Player 2")],
            player_selected: 0,
            mouse_action: None,
            ready: false,
        }
    }

    pub fn ready(&self) -> Option<Vec<String>> {
        if self.ready {
            Some(self.player_names.clone())
        } else {
            None
        }
    }

    fn valid_names(&self) -> bool {
        self.player_names.iter().all(|name| !name.is_empty())
    }

    fn current_player(&mut self) -> &mut String {
        self.player_names.get_mut(self.player_selected).unwrap()
    }

    // This function assumes that index is valid.
    fn select_player(&mut self, index: usize) {
        info!("Selecting player {}", index);
        self.player_selected = index;
    }

    // This function assumes that players.len() < 4.
    fn add_player(&mut self) {
        info!("Adding new player");
        let i = self.player_names.len();
        self.player_names.push(format!("Player {}", i + 1));
        self.player_selected = i;
    }

    // This function assumes that players.len() > 1 and the index is valid.
    fn remove_player(&mut self, index: usize) {
        info!("Removing player {}", index);
        self.player_names.remove(index);
        if self.player_selected >= index && self.player_selected > 0 {
            self.player_selected -= 1;
        }
    }

    // Add a character to player's name.
    fn add_character(&mut self, character: char) {
        let name = self.current_player();
        if name.len() < MAX_NAME_LEN {
            info!("Adding char {} to current player's name", character);
            name.push(character);
        }
    }

    // Remove last character of player's name. Used when Backspace is detected.
    fn remove_char(&mut self) {
        info!("Removing last char of current player's name");
        self.current_player().pop();
    }

    // Switch to next player. This function rotates index.
    fn next_player(&mut self) {
        self.player_selected += 1;
        if self.player_selected == self.player_names.len() {
            self.player_selected = 0;
        }
        info!("Selected player {}", self.player_selected);
    }

    // Switch to previous player. This function rotates index.
    fn prev_player(&mut self) {
        if self.player_selected == 0 {
            self.player_selected = self.player_names.len() - 1;
        } else {
            self.player_selected -= 1;
        }
        info!("Selected player {}", self.player_selected);
    }

    fn pos_to_button(&self, x: f32, y: f32) -> Option<ControlButton> {
        if x > LOBBY_PLAY_BUTTON_LEFT_CORNER.0
            && x <= LOBBY_PLAY_BUTTON_LEFT_CORNER.0 + LOBBY_PLAY_BUTTON_WIDTH
            && y > LOBBY_PLAY_BUTTON_LEFT_CORNER.1
            && y <= LOBBY_PLAY_BUTTON_LEFT_CORNER.1 + HUGE_FONT_SIZE + 2.0 * LOBBY_MARGIN
        {
            return Some(ControlButton::StartGame);
        }
        if x < LOBBY_ADD_REMOVE_BUTTON_X
            || x >= LOBBY_ADD_REMOVE_BUTTON_X + LOBBY_ROW_HEIGHT_WITH_MARGIN
            || y < LOBBY_FIRST_PLAYER_LEFT_CORNER.1
        {
            return None;
        }
        let uy = ((y - LOBBY_FIRST_PLAYER_LEFT_CORNER.1) / LOBBY_ROW_HEIGHT_WITH_MARGIN) as usize;
        match (self.player_names.len(), uy) {
            (1, 2) => Some(ControlButton::AddPlayer),
            (1, _) => None,
            (4, i) => {
                if i % 2 == 0 && i < 8 {
                    Some(ControlButton::RemovePlayer(i / 2))
                } else {
                    None
                }
            }
            (l, i) => {
                if i % 2 == 0 && i < l + l {
                    Some(ControlButton::RemovePlayer(i / 2))
                } else if i == l + l {
                    Some(ControlButton::AddPlayer)
                } else {
                    None
                }
            }
        }
    }

    fn pos_to_action(&self, x: f32, y: f32) -> Option<MouseAction> {
        self.pos_to_button(x, y)
            .map(MouseAction::ControlButton)
            .or_else(|| {
                if x < LOBBY_FIRST_PLAYER_LEFT_CORNER.0
                    || x > LOBBY_FIRST_PLAYER_LEFT_CORNER.0 + LOBBY_PLAYER_WIDTH
                {
                    return None;
                }
                let uy = ((y - LOBBY_FIRST_PLAYER_LEFT_CORNER.1) / LOBBY_ROW_HEIGHT_WITH_MARGIN)
                    as usize;
                match (self.player_names.len(), uy) {
                    (_, 0) => Some(MouseAction::PlayerName(0)),
                    (l, 2) if l >= 2 => Some(MouseAction::PlayerName(1)),
                    (l, 4) if l >= 3 => Some(MouseAction::PlayerName(2)),
                    (l, 6) if l >= 4 => Some(MouseAction::PlayerName(3)),
                    _ => None,
                }
            })
    }

    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult {
        // Part 1: Draw player names

        for i in 0..self.player_names.len() {
            // Draw rectangle (yellow background if selected, white otherwise)
            let x = LOBBY_FIRST_PLAYER_LEFT_CORNER.0;
            let y =
                LOBBY_FIRST_PLAYER_LEFT_CORNER.1 + (i + i) as f32 * LOBBY_ROW_HEIGHT_WITH_MARGIN;
            let w = LOBBY_PLAYER_WIDTH + 2.0 * LOBBY_MARGIN;
            let h = LOBBY_ROW_HEIGHT_WITH_MARGIN;

            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(x, y, w, h),
                if i == self.player_selected {
                    (190, 190, 0).into()
                } else {
                    (255, 255, 255).into()
                },
            )?
            .draw(ctx, graphics::DrawParam::default())?;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Stroke(
                    graphics::StrokeOptions::default().with_line_width(LOBBY_MARGIN),
                ),
                graphics::Rect::new(x, y, w, h),
                (0, 0, 0).into(),
            )?
            .draw(ctx, graphics::DrawParam::default())?;

            // Write name
            let mut name_text = graphics::Text::new(graphics::TextFragment {
                text: self.player_names[i].clone(),
                color: Some(graphics::Color::new(0.0, 0.0, 0.0, 1.0)),
                scale: Some(graphics::Scale::uniform(LARGE_FONT_SIZE)),
                ..Default::default()
            });
            name_text
                .set_bounds(
                    [LOBBY_PLAYER_WIDTH, LOBBY_ROW_HEIGHT],
                    graphics::Align::Left,
                )
                .draw(
                    ctx,
                    graphics::DrawParam::default().dest([x + LOBBY_MARGIN, y + LOBBY_MARGIN]),
                )?;
        }

        // Part 2: Draw add/remove buttons
        let mouse_pos = mouse::position(ctx);
        let button_hovered = if let Some(action) = self.mouse_action.clone() {
            match action {
                MouseAction::ControlButton(button) => Some(button),
                MouseAction::PlayerName(_) => None,
            }
        } else {
            self.pos_to_button(mouse_pos.x, mouse_pos.y)
        };
        let mut draw_button = |i: usize, hovered: bool, add: bool| -> GameResult {
            let x = LOBBY_ADD_REMOVE_BUTTON_X;
            let y =
                LOBBY_FIRST_PLAYER_LEFT_CORNER.1 + (i + i) as f32 * LOBBY_ROW_HEIGHT_WITH_MARGIN;
            let w = LOBBY_ROW_HEIGHT + LOBBY_MARGIN;
            let h = w;
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::Stroke(
                    graphics::StrokeOptions::default().with_line_width(LOBBY_MARGIN),
                ),
                graphics::Rect::new(x, y, w, h),
                if hovered {
                    (100, 100, 100).into()
                } else {
                    (0, 0, 0).into()
                },
            )?
            .draw(ctx, graphics::DrawParam::default())?;
            let img = if add {
                assets.add_player_image.clone()
            } else {
                assets.remove_player_image.clone()
            };
            img.draw(
                ctx,
                graphics::DrawParam::default()
                    .dest([x + LOBBY_MARGIN / 2.0, y + LOBBY_MARGIN / 2.0]),
            )
        };
        match self.player_names.len() {
            1 => draw_button(
                1,
                matches!(button_hovered, Some(ControlButton::AddPlayer)),
                true,
            )?,
            4 => {
                for i in 0..4 {
                    draw_button(
                        i,
                        matches!(button_hovered, Some(ControlButton::RemovePlayer(j)) if i == j),
                        false,
                    )?;
                }
            }
            l => {
                for i in 0..l {
                    draw_button(
                        i,
                        matches!(button_hovered, Some(ControlButton::RemovePlayer(j)) if i == j),
                        false,
                    )?;
                }
                draw_button(
                    l,
                    matches!(button_hovered, Some(ControlButton::AddPlayer)),
                    true,
                )?;
            }
        }

        // Part 3: Draw "Play" button
        let x = LOBBY_PLAY_BUTTON_LEFT_CORNER.0;
        let y = LOBBY_PLAY_BUTTON_LEFT_CORNER.1;
        let w = LOBBY_PLAY_BUTTON_WIDTH;
        let h = HUGE_FONT_SIZE + 2.0 * LOBBY_MARGIN;

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
                graphics::StrokeOptions::default().with_line_width(LOBBY_MARGIN),
            ),
            graphics::Rect::new(x, y, w, h),
            if matches!(button_hovered, Some(ControlButton::StartGame)) {
                if self.valid_names() {
                    (100, 100, 100).into()
                } else {
                    (255, 0, 0).into()
                }
            } else {
                (0, 0, 0).into()
            },
        )?
        .draw(ctx, graphics::DrawParam::default())?;
        let mut play_text = graphics::Text::new(graphics::TextFragment {
            text: "Play".to_owned(),
            color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
            scale: Some(graphics::Scale::uniform(HUGE_FONT_SIZE)),
            ..Default::default()
        });
        play_text
            .set_bounds(
                [w - 2.0 * LOBBY_MARGIN, h - 2.0 * LOBBY_MARGIN],
                graphics::Align::Center,
            )
            .draw(
                ctx,
                graphics::DrawParam::default().dest([x + LOBBY_MARGIN, y + LOBBY_MARGIN]),
            )
    }

    pub fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if button != MouseButton::Left {
            return;
        }
        self.mouse_action = self.pos_to_action(x, y);
    }

    pub fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if button != MouseButton::Left {
            return;
        }
        if self.mouse_action == self.pos_to_action(x, y) {
            if let Some(action) = self.mouse_action.clone() {
                match action {
                    MouseAction::ControlButton(button) => match button {
                        ControlButton::AddPlayer => self.add_player(),
                        ControlButton::RemovePlayer(index) => self.remove_player(index),
                        ControlButton::StartGame => self.ready = self.valid_names(),
                    },
                    MouseAction::PlayerName(index) => self.select_player(index),
                }
            }
        }
        self.mouse_action = None;
    }

    pub fn text_input_event(&mut self, _ctx: &mut Context, character: char) {
        if !character.is_control() {
            self.add_character(character)
        }
    }

    pub fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        match keycode {
            KeyCode::Up => self.prev_player(),
            KeyCode::Down => self.next_player(),
            KeyCode::Back => self.remove_char(),
            _ => {}
        }
    }
}
