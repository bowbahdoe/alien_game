use crate::bullet::{Bullet, BulletFactory};
use ggez::graphics::{DrawParam, Drawable};
use ggez::mint::Point2;
use ggez::{graphics, Context, GameResult};
use rand::Rng;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone)]
struct MovementPlan {
    start_pos: (f32, f32),
    next_pos: (f32, f32),
    plan_made: Instant,
    duration: Duration,
}

#[derive(Debug, Copy, Clone)]
struct FiringPlan {
    dangerous: bool,
    plan_made: Instant,
    delay: Duration,
}

pub struct Alien {
    pos: (f32, f32),
    x_movement_range: (f32, f32),
    idle: Rc<graphics::Image>,
    firing: Rc<graphics::Image>,
    firing_plan: Option<FiringPlan>,
    movement_plan: Option<MovementPlan>,
}

impl Debug for Alien {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Alien {{ pos: {:?}, ... }}", self.pos)
    }
}

impl Alien {
    pub fn pos(&self) -> (f32, f32) {
        self.pos
    }

    pub fn starting_at(
        pos: (f32, f32),
        x_movement_range: (f32, f32),
        idle: Rc<graphics::Image>,
        firing: Rc<graphics::Image>,
    ) -> Alien {
        Alien {
            pos,
            x_movement_range,
            idle,
            firing,
            movement_plan: None,
            firing_plan: None,
        }
    }

    fn will_fire(&self, now: Instant) -> bool {
        if let Some(plan) = self.firing_plan {
            plan.plan_made + plan.delay < now
        } else {
            false
        }
    }

    pub fn update(
        &mut self,
        now: Instant,
        bullet_factory: &mut impl BulletFactory,
    ) -> Option<Bullet> {
        let mut rng = rand::thread_rng();

        let (min_x, max_x) = self.x_movement_range;
        let start_pos = self.pos;
        let mut gen_movement_plan = || MovementPlan {
            start_pos,
            next_pos: (rng.gen_range(min_x, max_x), start_pos.1),
            plan_made: now,
            duration: Duration::from_millis(rand::thread_rng().gen_range(300, 2000)),
        };

        match &self.movement_plan {
            &None => self.movement_plan = Some(gen_movement_plan()),
            Some(plan) => {
                if plan.plan_made + plan.duration < now {
                    self.pos = plan.next_pos;
                    self.movement_plan = Some(gen_movement_plan())
                } else {
                    let tween_pct =
                        (now - plan.plan_made).as_secs_f32() / plan.duration.as_secs_f32();
                    self.pos = (
                        plan.start_pos.0 + tween_pct * (plan.next_pos.0 - plan.start_pos.0),
                        plan.start_pos.1 + tween_pct * (plan.next_pos.1 - plan.start_pos.1),
                    )
                }
            }
        }

        let mut fired = None;
        match self.firing_plan {
            None => {
                self.firing_plan = Some(FiringPlan {
                    dangerous: false,
                    plan_made: now,
                    delay: Duration::from_millis(1000),
                })
            }
            Some(plan) => {
                if plan.plan_made + plan.delay < now {
                    if plan.dangerous {
                        fired = Some(bullet_factory.red_bullet(self.pos));
                    } else {
                        fired = Some(bullet_factory.green_bullet(self.pos));
                    }
                    self.firing_plan = Some(FiringPlan {
                        dangerous: rand::thread_rng().gen_bool(0.2),
                        plan_made: now,
                        delay: Duration::from_millis(rand::thread_rng().gen_range(200, 700)),
                    })
                }
            }
        }

        fired
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let sprite = if self.will_fire(Instant::now() + Duration::from_millis(200)) {
            &self.firing
        } else {
            &self.idle
        };
        sprite.draw(
            ctx,
            DrawParam::default()
                .offset(Point2{x: 0.5, y: 0.5})
                .dest(Point2{x: self.pos.0, y: self.pos.1}),
        )
    }
}
