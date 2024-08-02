use ggez::audio::SoundSource;
use ggez::event::{self, EventHandler};
use ggez::graphics::{Canvas, DrawParam, Drawable, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::Point2;
use ggez::{audio, graphics, Context, ContextBuilder, GameResult};
use std::collections::HashSet;
use std::env;
use std::f32::consts::FRAC_PI_2;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path;
use std::rc::Rc;
use std::time::{Duration, Instant};

mod alien;
mod bullet;
mod simple_collision;

use crate::alien::Alien;
use crate::bullet::{Bullet, BulletFactoryImpl};
use crate::simple_collision::{are_colliding, CollisionRect};

const _GAME_OVER_MESSAGES: [&str; 13] = [
    "You lost",
    "You can do better than that",
    "Catch Them!",
    "Better luck next time",
    "Mwahahahaha",
    "Be better than that",
    "Good job...?",
    "Failed You Have",
    "Nice try",
    "Get a different hobby",
    "You missed a spot",
    "Great job",
    "Wax on. Wax off.",
];

#[derive(Debug)]
struct Player {
    pos: (f32, f32),
    sprite: Rc<graphics::Image>,
}

impl CollisionRect for Player {
    fn top_left_x(&self) -> f32 {
        self.pos.0 - self.sprite.width() as f32 / 2.0
    }

    fn top_left_y(&self) -> f32 {
        self.pos.1 - self.sprite.height() as f32 / 2.0
    }

    fn width(&self) -> f32 {
        self.sprite.width() as f32
    }

    fn height(&self) -> f32 {
        self.sprite.height() as f32
    }
}

impl Player {
    fn starting_at(pos: (f32, f32), sprite: Rc<graphics::Image>) -> Player {
        Player { pos, sprite }
    }

    const PLAYER_SPEED: f32 = 1000.0;

    fn execute_intent(&mut self, player_intent: &PlayerIntent, dt: Duration) {
        match *player_intent {
            PlayerIntent::StayStill => {}
            PlayerIntent::MoveLeft => {
                self.pos = (
                    self.pos.0 - Player::PLAYER_SPEED * dt.as_secs_f32(),
                    self.pos.1,
                )
            }
            PlayerIntent::MoveRight => {
                self.pos = (
                    self.pos.0 + Player::PLAYER_SPEED * dt.as_secs_f32(),
                    self.pos.1,
                )
            }
        }
    }
}

#[derive(Debug, Default)]
enum PlayerIntent {
    MoveLeft,
    MoveRight,
    #[default]
    StayStill,
}

struct SpriteData {
    alien_idle: Rc<graphics::Image>,
    alien_firing: Rc<graphics::Image>,
    player: Rc<graphics::Image>,
    red_bullet: Rc<graphics::Image>,
    green_bullet: Rc<graphics::Image>,
    background: Rc<graphics::Image>,
}

impl SpriteData {
    fn load_from_resources(ctx: &mut Context) -> ggez::GameResult<SpriteData> {
        Ok(SpriteData {
            alien_idle: Rc::new(graphics::Image::from_path(ctx, "/ENEMY.png")?),
            alien_firing: Rc::new(graphics::Image::from_path(ctx, "/ENEMY_FIRING.png")?),
            player: Rc::new(graphics::Image::from_path(ctx, "/PLAYER_OLD_2.png")?),
            red_bullet: Rc::new(graphics::Image::from_path(ctx, "/Red_Missile.png")?),
            green_bullet: Rc::new(graphics::Image::from_path(ctx, "/MISSILE_FIRED.png")?),
            background: Rc::new(graphics::Image::from_path(ctx, "/Space.png")?),
        })
    }
}

impl Debug for SpriteData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SpriteData {{...}}")
    }
}

struct AudioData {
    bloop: audio::Source,
}

impl AudioData {
    fn load_from_resources(ctx: &mut Context) -> ggez::GameResult<AudioData> {
        Ok(AudioData {
            bloop: audio::Source::new(ctx, "/Bloop.ogg")?,
        })
    }
}

impl Debug for AudioData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AudioData {{...}}")
    }
}

#[derive(Debug, Default)]
struct KeysPressed {
    left: bool,
    right: bool,
}

#[derive(Debug)]
struct Game {
    alien: Alien,
    player: Player,
    bullets: Vec<Bullet>,
    last_tick: Instant,
    score: u32,
    game_over: bool,
    screen_size: (u32, u32),
    keys_pressed: KeysPressed,
    sprites: SpriteData,
    audio: AudioData,
}

impl Game {
    fn starting(ctx: &mut Context) -> ggez::GameResult<Game> {
        let canvas = Canvas::from_frame(ctx, None);
        let screen_coordinates = canvas.screen_coordinates().unwrap();
        let sprites = SpriteData::load_from_resources(ctx)?;
        Ok(Game {
            alien: Alien::starting_at(
                (50.0, 50.0),
                (0.0, screen_coordinates.w),
                sprites.alien_idle.clone(),
                sprites.alien_firing.clone(),
            ),
            player: Player::starting_at((50.0, 550.0), sprites.player.clone()),
            bullets: vec![],
            last_tick: Instant::now(),
            score: 0,
            game_over: false,
            screen_size: (screen_coordinates.w as u32, screen_coordinates.h as u32),
            keys_pressed: KeysPressed::default(),
            sprites,
            audio: AudioData::load_from_resources(ctx)?,
        })
    }

    fn player_intent(&self) -> PlayerIntent {
        if self.keys_pressed.left && self.keys_pressed.right {
            PlayerIntent::StayStill
        } else if self.keys_pressed.left {
            PlayerIntent::MoveLeft
        } else if self.keys_pressed.right {
            PlayerIntent::MoveRight
        } else {
            PlayerIntent::StayStill
        }
    }
}

fn distance(p1: &(f32, f32), p2: &(f32, f32)) -> f32 {
    ((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()
}

fn clean_up_out_of_bounds_bullets(game: &mut Game) {
    let screen_area = (game.screen_size.0 * game.screen_size.1) as f32;
    game.bullets.retain(|bullet| {
        // Delete them once they are very far away.
        // TODO: Replace with more sensitive checking for score keeping.
        distance(&bullet.pos(), &(0.0, 0.0)) < screen_area
    });
}

fn tick(ctx: &mut Context, game: &mut Game, dt: Duration) -> GameResult<()> {
    let now = game.last_tick + dt;

    game.bullets
        .iter_mut()
        .for_each(|bullet| bullet.move_down(dt));
    if let Some(new_bullet) = game.alien.update(
        now,
        &mut BulletFactoryImpl {
            green_sprite: &game.sprites.green_bullet,
            red_sprite: &game.sprites.red_bullet,
        },
    ) {
        game.bullets.push(new_bullet);
    }

    let mut keep = HashSet::new();
    for (idx, bullet) in game.bullets.iter().enumerate() {
        if are_colliding(&game.player, bullet) {
            if bullet.deadly() {
                game.game_over = true;
            } else {
                game.score += 1;
            }
            game.audio.bloop.play(ctx)?;
        } else {
            keep.insert(idx);
        }
    }
    let bullets = &mut game.bullets;
    let player = &game.player;
    bullets.retain(|bullet| !are_colliding(player, bullet));

    game.player.execute_intent(&game.player_intent(), dt);
    clean_up_out_of_bounds_bullets(game);

    game.last_tick = now;
    Ok(())
}

fn draw_background(canvas: &mut Canvas, game: &Game) {
    game.sprites.background.draw(canvas, DrawParam::default())
}

fn draw_bullets(canvas: &mut Canvas, game: &Game) {
    for bullet in game.bullets.iter() {
        bullet.draw(canvas);
    }
}

fn draw_enemy(canvas: &mut Canvas, game: &Game) {
    game.alien.draw(canvas)
}

fn draw_player(canvas: &mut Canvas, game: &Game) {
    game.sprites.player.draw(
        canvas,
        DrawParam::default()
            .offset(Point2 { x: 0.5, y: 0.5 })
            .dest(Point2 {
                x: game.player.pos.0,
                y: game.player.pos.1,
            })
            .rotation(FRAC_PI_2),
    );
}

fn draw_score(canvas: &mut Canvas, game: &Game) {
    let text = Text::new(format!("{}", game.score));
    text.draw(
        canvas,
        DrawParam::default().dest(Point2 {
            x: game.screen_size.0 as f32 / 2.0,
            y: game.screen_size.1 as f32 / 2.0,
        }),
    );
}

impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = Instant::now() - self.last_tick;
        tick(ctx, self, dt)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        draw_background(&mut canvas, self);
        draw_bullets(&mut canvas, self);
        draw_enemy(&mut canvas, self);
        draw_player(&mut canvas, self);
        draw_score(&mut canvas, self);
        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::Escape) => ctx.request_quit(),
            Some(KeyCode::Left) => {
                self.keys_pressed.left = true;
            }
            Some(KeyCode::Right) => {
                self.keys_pressed.right = true;
            }
            _ => (), // Do nothing
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        match input.keycode {
            Some(KeyCode::Left) => {
                self.keys_pressed.left = false;
            }
            Some(KeyCode::Right) => {
                self.keys_pressed.right = false;
            }
            _ => (), // Do nothing
        }
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.screen_size = (width as u32, height as u32);
        Ok(())
    }
}

fn main() -> GameResult {
    let mut ctx_builder = ContextBuilder::new("kablewey", "Ethan McCue");

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx_builder = ctx_builder.add_resource_path(path);
    }

    let (mut ctx, event_loop) = ctx_builder.build()?;

    let my_game = Game::starting(&mut ctx)?;
    event::run(ctx, event_loop, my_game)
}
