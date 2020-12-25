use crate::active_game::ActiveGame;
use crate::assets::Assets;
use crate::lobby::Lobby;
use crate::summary::Summary;
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::{graphics, input::mouse::MouseButton, timer, Context, GameResult};
use log::info;

pub enum GameStatus {
    ActiveGame(ActiveGame),
    Lobby(Lobby),
    Summary(Summary),
}

impl GameStatus {
    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        match self {
            GameStatus::ActiveGame(game) => game.draw(ctx, assets),
            GameStatus::Lobby(lobby) => lobby.draw(ctx, assets),
            GameStatus::Summary(summary) => summary.draw(ctx, assets),
        }
    }
}

pub struct Game {
    assets: Assets,
    game_status: GameStatus,
    mouse_coords: (f32, f32),
}

impl Game {
    pub fn new(ctx: &mut Context) -> GameResult<Game> {
        let mouse_pos = ggez::input::mouse::position(ctx);
        Ok(Game {
            assets: Assets::new(ctx)?,
            game_status: GameStatus::Lobby(Lobby::new()),
            mouse_coords: (mouse_pos.x, mouse_pos.y),
        })
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        match &mut self.game_status {
            GameStatus::ActiveGame(game) => {
                if let Some(finish) = game.check_finish() {
                    self.game_status = GameStatus::Summary(Summary::new(ctx, finish)?);
                }
            }
            GameStatus::Lobby(lobby) => {
                if let Some(player_names) = lobby.ready() {
                    self.game_status = GameStatus::ActiveGame(ActiveGame::new(player_names))
                }
            }
            GameStatus::Summary(summary) => {
                if summary.is_finished() {
                    self.game_status = GameStatus::Lobby(Lobby::new());
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, (130, 60, 20).into()); // Brown background
        self.game_status.draw(ctx, &self.assets)?;
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        info!(
            "Registered mouse button down event: ({:?}, {}, {}).",
            button, x, y
        );
        match &mut self.game_status {
            GameStatus::ActiveGame(game) => game.mouse_button_down_event(ctx, button, x, y),
            GameStatus::Lobby(lobby) => lobby.mouse_button_down_event(ctx, button, x, y),
            GameStatus::Summary(summary) => summary.mouse_button_down_event(ctx, button, x, y),
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        info!(
            "Registered mouse button up event: ({:?}, {}, {}).",
            button, x, y
        );
        match &mut self.game_status {
            GameStatus::ActiveGame(game) => game.mouse_button_up_event(ctx, button, x, y),
            GameStatus::Lobby(lobby) => lobby.mouse_button_up_event(ctx, button, x, y),
            GameStatus::Summary(summary) => summary.mouse_button_up_event(ctx, button, x, y),
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.mouse_coords = (x, y);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        info!("Registered key down: {:?}.", keycode);
        match &mut self.game_status {
            GameStatus::ActiveGame(_) => {}
            GameStatus::Lobby(lobby) => lobby.key_down_event(ctx, keycode, keymods, repeat),
            GameStatus::Summary(_) => {}
        }
    }

    fn text_input_event(&mut self, ctx: &mut Context, character: char) {
        info!("Registered text input event: {}.", character);
        match &mut self.game_status {
            GameStatus::ActiveGame(_) => {}
            GameStatus::Lobby(lobby) => lobby.text_input_event(ctx, character),
            GameStatus::Summary(_) => {}
        }
    }
}
