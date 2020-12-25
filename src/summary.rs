use crate::{assets::Assets, consts::*, highscores::Highscores};
use ggez::{
    event::MouseButton,
    graphics::{self, Drawable},
    Context, GameResult,
};
use log::info;

pub struct Summary {
    summary_headline: graphics::Text,
    highscores_headline: graphics::Text,
    results: Vec<graphics::Text>,
    footer: graphics::Text,
    highscores: Option<Vec<graphics::Text>>,
    clicking: bool,
    finished: bool,
}

impl Summary {
    // This function generates all assets necessary to draw game summary.
    // Since building new graphics::Text every frame is expensive and the text
    // is always the same, the function creates graphics::Text and stores them.
    pub fn new(ctx: &mut Context, mut results: Vec<(String, u64)>) -> GameResult<Summary> {
        info!("The game has been finished, generating summary.");
        let mut summary_headline = graphics::Text::new(graphics::TextFragment {
            text: SUMMARY_HEADLINE.to_string(),
            color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
            scale: Some(graphics::Scale::uniform(FONT_SIZE)),
            ..Default::default()
        });
        summary_headline.set_bounds(
            [SUMMARY_TEXT_SIZE.0, SUMMARY_TEXT_SIZE.1],
            graphics::Align::Left,
        );

        let mut highscores_headline = graphics::Text::new(graphics::TextFragment {
            text: HIGHSCORES_HEADLINE.to_string(),
            color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
            scale: Some(graphics::Scale::uniform(FONT_SIZE)),
            ..Default::default()
        });
        highscores_headline.set_bounds(
            [SUMMARY_TEXT_SIZE.0, SUMMARY_TEXT_SIZE.1],
            graphics::Align::Left,
        );

        results.sort_by_key(|result| result.1);
        results.reverse();
        let result_texts = results
            .iter()
            .cloned()
            .map(|(name, score)| {
                info!("Player {} scored {} points.", name, score);
                let mut text = graphics::Text::new(graphics::TextFragment {
                    text: format!("{} ({})", name, score),
                    color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
                    scale: Some(graphics::Scale::uniform(FONT_SIZE)),
                    ..Default::default()
                });
                text.set_bounds(
                    [SUMMARY_TEXT_SIZE.0, SUMMARY_TEXT_SIZE.1],
                    graphics::Align::Left,
                );
                text
            })
            .collect();

        let mut footer = graphics::Text::new(graphics::TextFragment {
            text: "Click anywhere to continue".to_string(),
            color: Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0)),
            scale: Some(graphics::Scale::uniform(SMALL_FONT_SIZE)),
            ..Default::default()
        });
        footer.set_bounds(
            [SUMMARY_TEXT_SIZE.0, SMALL_FONT_SIZE],
            graphics::Align::Center,
        );

        // Highscores
        let mut highscores = Highscores::new(ctx)?;
        let mut highscores_changed = false;
        for (player_name, score) in results {
            highscores_changed |= highscores.update(player_name, score);
        }
        let highscore_texts = if highscores_changed {
            Some(
                highscores
                    .entries()
                    .iter()
                    .cloned()
                    .map(|entry| {
                        let color = if entry.is_new() {
                            Some(graphics::Color::new(1.0, 0.0, 0.0, 1.0))
                        } else {
                            Some(graphics::Color::new(1.0, 1.0, 1.0, 1.0))
                        };
                        let mut text = graphics::Text::new(graphics::TextFragment {
                            text: format!("{} ({})", entry.name, entry.score),
                            color,
                            scale: Some(graphics::Scale::uniform(FONT_SIZE)),
                            ..Default::default()
                        });
                        text.set_bounds(
                            [SUMMARY_TEXT_SIZE.0, SUMMARY_TEXT_SIZE.1],
                            graphics::Align::Left,
                        );
                        text
                    })
                    .collect(),
            )
        } else {
            None
        };
        highscores.save(ctx)?;

        Ok(Summary {
            summary_headline,
            highscores_headline,
            results: result_texts,
            footer,
            highscores: highscore_texts,
            clicking: false,
            finished: false,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn draw(&self, ctx: &mut Context, _assets: &Assets) -> GameResult {
        let (headline, scores) = if let Some(highscores) = &self.highscores {
            (&self.highscores_headline, highscores)
        } else {
            (&self.summary_headline, &self.results)
        };
        headline.draw(
            ctx,
            graphics::DrawParam::default().dest([
                SUMMARY_HEADLINE_LEFT_CORNER.0,
                SUMMARY_HEADLINE_LEFT_CORNER.1,
            ]),
        )?;
        for (i, text) in scores.iter().enumerate() {
            text.draw(
                ctx,
                graphics::DrawParam::default().dest([
                    SUMMARY_HEADLINE_LEFT_CORNER.0,
                    SUMMARY_HEADLINE_LEFT_CORNER.1 + (i + 1) as f32 * SUMMARY_TEXT_ROW_SIZE,
                ]),
            )?;
        }
        self.footer.draw(
            ctx,
            graphics::DrawParam::default()
                .dest([SUMMARY_HEADLINE_LEFT_CORNER.0, SUMMARY_TEXT_FOOTER_Y]),
        )
    }

    pub fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button != MouseButton::Left {
            return;
        }
        self.clicking = true;
    }

    pub fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        if button != MouseButton::Left {
            return;
        }
        if self.clicking {
            if self.highscores.is_some() {
                self.highscores = None;
            } else {
                self.finished = true;
            }
        }
        self.clicking = false;
    }
}
