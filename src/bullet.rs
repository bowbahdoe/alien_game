use crate::simple_collision::CollisionRect;
use ggez::graphics::{DrawParam, Drawable};
use ggez::nalgebra::Point2;
use ggez::{graphics, Context, GameResult};
use std::f32::consts::FRAC_PI_2;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone)]
pub struct Bullet {
    pos: (f32, f32),
    color: BulletColor,
    sprite: Rc<graphics::Image>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum BulletColor {
    Red,
    Green,
}

impl Debug for Bullet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bullet {{ pos: {:?}, color: {:?}, sprite: ... }}",
            self.pos, self.color
        )
    }
}

impl Bullet {
    const VELOCITY: f32 = 500.0;

    pub fn pos(&self) -> (f32, f32) {
        self.pos
    }

    pub fn move_down(&mut self, time_passed: Duration) {
        self.pos = (
            self.pos.0,
            self.pos.1 + Bullet::VELOCITY * time_passed.as_secs_f32(),
        )
    }

    pub fn deadly(&self) -> bool {
        self.color != BulletColor::Green
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.sprite.draw(
            ctx,
            DrawParam::default()
                .offset(Point2::new(0.5, 0.5))
                .dest(Point2::new(self.pos.0, self.pos.1))
                .rotation(FRAC_PI_2),
        )
    }
}

/// A factory for producing bullets so as to minimize cloning of underlying image resources
/// and not have that detail leak to consumers.
pub trait BulletFactory {
    /// Produces a red bullet. Not safe to catch.
    fn red_bullet(&mut self, pos: (f32, f32)) -> Bullet;
    /// Produces a green bullet. Not safe to not catch.
    fn green_bullet(&mut self, pos: (f32, f32)) -> Bullet;
}

/// Simple bullet factory that keeps references to some Rcs to avoid extra clones.
pub struct BulletFactoryImpl<'a> {
    /// The sprite to use for a red bullet.
    pub red_sprite: &'a Rc<graphics::Image>,
    /// The sprite to use for a green bullet.
    pub green_sprite: &'a Rc<graphics::Image>,
}

impl BulletFactory for BulletFactoryImpl<'_> {
    fn red_bullet(&mut self, pos: (f32, f32)) -> Bullet {
        Bullet {
            pos,
            color: BulletColor::Red,
            sprite: self.red_sprite.clone(),
        }
    }

    fn green_bullet(&mut self, pos: (f32, f32)) -> Bullet {
        Bullet {
            pos,
            color: BulletColor::Green,
            sprite: self.green_sprite.clone(),
        }
    }
}

impl CollisionRect for Bullet {
    fn top_left_x(&self) -> f32 {
        self.pos.0 - self.sprite.dimensions().w as f32 / 2.0
    }

    fn top_left_y(&self) -> f32 {
        self.pos.1 - self.sprite.dimensions().h as f32 / 2.0
    }

    fn width(&self) -> f32 {
        self.sprite.dimensions().w
    }

    fn height(&self) -> f32 {
        self.sprite.dimensions().h
    }
}
