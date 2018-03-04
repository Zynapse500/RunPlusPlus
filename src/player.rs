use trap::Vector2;
use rax::collision::*;

use rax::Renderer;

pub struct Player {
    collision: ConvexHull,

    velocity: Vector2,
    move_direction: Option<MoveDirection>,

    ground_normal: Option<Vector2>,
    wall_normal: Option<Vector2>,

    jumping: bool,

    commands: Vec<PlayerCommand>,
}


#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PlayerCommand {
    MoveLeft,
    MoveRight,

    Jump,
    StopJump,

    Drop,
}


#[derive(Eq, PartialEq, Copy, Clone)]
enum MoveDirection {
    Left,
    Right,
}


impl Player {
    pub fn new(position: Vector2) -> Player {
        let w = 24.0;
        let h = 48.0;
        Player {
            collision: ConvexHull::from_points(&[
                Vector2 { x: position.x - w / 2.0, y: position.y - h / 2.0 },
                Vector2 { x: position.x + w / 2.0, y: position.y - h / 2.0 },
                Vector2 { x: position.x + w / 2.0, y: position.y + h / 2.0 },
                Vector2 { x: position.x - w / 2.0, y: position.y + h / 2.0 },
            ]),

            velocity: Vector2 { x: 0.0, y: 0.0 },
            move_direction: None,

            ground_normal: None,
            wall_normal: None,
            jumping: false,

            commands: Vec::new(),
        }
    }


    /// Update the player's position and movement
    pub fn update(&mut self, dt: f64, obstacles: &[&Collide<ConvexHull>]) {
        self.handle_commands(dt);

        self.velocity.x -= self.velocity.x * dt * 1.0;
        self.velocity.y -= self.velocity.y * dt * 0.5;

        if let Some(normal) = self.wall_normal {
            let dot = normal.dot(Vector2::new(-1.0, 0.0));
            if dot < -0.95 ||
                dot > 0.95 {
                if self.velocity.y > 0.0 {
                    self.velocity.y -= self.velocity.y * dt * 9.0;
                }
            }
        }

        if self.jumping {
            if let Some(normal) = self.ground_normal {
                self.velocity += normal * 250.0;
                self.jumping = false;
                self.wall_normal = None;
            } else if let Some(normal) = self.wall_normal {
                self.velocity += (normal + Vector2::new(0.0, -1.5)).norm() * 250.0;
                self.jumping = false;
                self.wall_normal = None;
            }
        }

        if self.velocity.y > 0.0 {
            self.velocity.y += 400.0 * dt;
        } else {
            self.velocity.y += 200.0 * dt;
        }

        let amount = self.velocity * dt;
        self.translate(amount);

        self.check_collisions(obstacles);

    }


    /// Draw the player
    pub fn draw(&self, renderer: &mut Renderer) {
        {
            renderer.color = [0.0, 0.0, 1.0, 0.3];
            renderer.fill_convex(self.collision.get_points());

            let bounds = self.collision.bounding_box();
            let left = bounds.left;
            let right = bounds.right;
            let top = bounds.top;
            let bottom = bounds.bottom;

            let mid = Vector2::new((left + right) / 2.0, (top + bottom) / 2.0);
            let size = Vector2::new(right - left, bottom - top);

            renderer.color = [1.0, 1.0, 1.0, 1.0];
            renderer.line_width = 2.0;

            // Legs
            {
                let mut hip = mid;

                let (mut contact_leg_left, mut contact_leg_right, mut contact_arm_left, mut contact_arm_right) = if let Some(normal) = self.wall_normal {
                    let x = mid.x - normal.x * size.x / 2.0;

                    hip.x = mid.x - normal.x * size.x / 4.0;

                    (
                        Vector2::new(x, bottom - size.y / 5.0),
                        Vector2::new(x, bottom),
                        Vector2::new(x, top + size.y / 3.0),
                        Vector2::new(x, top + size.y / 2.0),
                    )
                } else if let Some(normal) = self.ground_normal {
                    let angle = (hip.x / size.x * 1.0).sin() / 3.0;
                    let contact_x = if normal.x < 0.0 { right } else { left };

                    let left_x = hip.x + angle * size.x;
                    let right_x = hip.x - angle * size.x;

                    let slope = normal.x / normal.y;

                    (
                        Vector2::new(left_x, bottom + (contact_x - left_x) * slope),
                        Vector2::new(right_x, bottom + (contact_x - right_x) * slope),
                        Vector2::new(mid.x + angle * size.x / 2.0, mid.y),
                        Vector2::new(mid.x - angle * size.x / 2.0, mid.y),
                    )
                } else {
                    let x = hip.x - self.velocity.x * size.x / 2.0 / 250.0;
                    (
                        Vector2::new(x, bottom - size.y / 5.0),
                        Vector2::new(x, bottom),
                        Vector2::new(x + size.x / 3.0, mid.y - size.y / 5.0),
                        Vector2::new(x - size.x / 3.0, mid.y),
                    )
                };

                let leg_length = size.y / 1.5;
                let arm_length = size.y / 3.0;

                let joint = |contact: &mut Vector2, length: f64| {
                    let delta = hip - *contact;
                    let distance = delta.len();
                    let discriminant = (length / 2.0).powi(2) - (distance / 2.0).powi(2);
                    let middle = (hip + *contact) / 2.0;
                    if discriminant < 0.0 {
                        *contact = hip + (*contact - middle).norm() * length;
                        middle
                    } else {
                        let advance = discriminant.sqrt();
                        let direction = if self.velocity.x < 0.0 {
                            -1.0
                        } else if self.velocity.x > 0.0 {
                            1.0
                        } else {
                            if let Some(normal) = self.wall_normal {
                                if normal.x > 0.0 {
                                    1.0
                                } else {
                                    -1.0
                                }
                            } else {
                                0.5
                            }
                        };
                        middle + Vector2::new(-delta.y, delta.x).norm() * advance * direction
                    }
                };

                let leg_joint_left = joint(&mut contact_leg_left, leg_length);
                let leg_joint_right = joint(&mut contact_leg_right, leg_length);
                let arm_joint_left = joint(&mut contact_arm_left, arm_length);
                let arm_joint_right = joint(&mut contact_arm_right, arm_length);

                // Legs
                renderer.draw_line(hip, leg_joint_left);
                renderer.draw_line(hip, leg_joint_right);

                renderer.draw_line(leg_joint_left, contact_leg_left);
                renderer.draw_line(leg_joint_right, contact_leg_right);

                let upper = Vector2::new(mid.x, mid.y - size.y / 4.0);

                // Arms
                renderer.draw_line(upper, arm_joint_left);
                renderer.draw_line(upper, arm_joint_right);

                renderer.draw_line(arm_joint_left, contact_arm_left);
                renderer.draw_line(arm_joint_right, contact_arm_right);

                // Chest
                renderer.draw_line(upper, Vector2::new(hip.x, hip.y));
            }
        }
    }


    /// Make the player execute a command
    pub fn submit_command(&mut self, command: PlayerCommand) {
        self.commands.push(command);
    }


    /// Translate the player
    pub fn translate(&mut self, amount: Vector2) {
        self.collision.translate(amount);
    }


    /// Handle any commands
    fn handle_commands(&mut self, dt: f64) {
        self.move_direction = None;

        let commands = self.commands.clone();
        for command in commands {
            match command {
                PlayerCommand::MoveLeft => { self.move_direction(MoveDirection::Left, dt); }
                PlayerCommand::MoveRight => { self.move_direction(MoveDirection::Right, dt); }
                PlayerCommand::Jump => { self.jumping = true; }
                PlayerCommand::StopJump => { self.jumping = false; }
                PlayerCommand::Drop => { self.wall_normal = None; }
            }
        }

        self.commands.clear();
    }


    /// Make the player move in a direction
    fn move_direction(&mut self, direction: MoveDirection, dt: f64) {
        if self.wall_normal.is_none() {
            let plane = if let Some(normal) = self.ground_normal {
                Vector2::new(-normal.y, normal.x)
            } else {
                Vector2::new(1.0, 0.0)
            };
            let delta = if direction == MoveDirection::Left { -plane } else { plane };

            self.velocity += delta * 300.0 * dt * if self.ground_normal.is_some() { 1.0 } else { 0.75 };

            if self.move_direction.is_none() {
                self.move_direction = Some(direction);
            } else if let Some(dir) = self.move_direction.clone() {
                if dir != direction {
                    self.move_direction = None;
                }
            }
        }
    }


    /// Check for and resolve any collisions
    fn check_collisions(&mut self, obstacles: &[&Collide<ConvexHull>]) {
        self.ground_normal = None;

        let mut i = 0;
        loop {
            let first = {
                // First, find all overlaps, then find the smallest overlap
                obstacles.iter().filter_map(|o| { o.overlap(&self.collision) })
                    .min_by(|a, b| { a.0.partial_cmp(&b.0).unwrap() })
            };

            if let Some(overlap) = first {
                self.resolve_overlap(overlap);
                i += 1;
                if i > 100 {
                    break;
                }
            } else {
                break;
            }
        }


        self.check_wall_climb(obstacles);
    }

    /// Resolves an overlap
    fn resolve_overlap(&mut self, overlap: (f64, Vector2)) {
        let (_, resolve) = overlap;

        self.translate(-resolve);

        let normal = -resolve.norm();

        // Slide
        if normal.dot(self.velocity) < 0.0 {
            let plane = Vector2::new(normal.y, -normal.x);
            self.velocity = plane * plane.dot(self.velocity);
        }


        // Check if grounded or wall sliding
        if normal.dot(Vector2::new(0.0, -1.0)) > 0.5 {
            self.ground_normal = Some(normal);
            self.wall_normal = None;
        } else {
            let dot = normal.dot(Vector2::new(0.0, -1.0));
            if dot < 0.05 && dot > -0.05 {
                self.wall_normal = Some(normal);
            }
        }
    }


    fn check_wall_climb(&mut self, obstacles: &[&Collide<ConvexHull>]) {
        if let Some(normal) = self.wall_normal {
            let delta = -normal;
            self.translate(delta);
            let first = {
                // First, find all overlaps, then find the smallest overlap
                obstacles.iter().filter_map(|o| { o.overlap(&self.collision) })
                    .min_by(|a, b| { a.0.partial_cmp(&b.0).unwrap() })
            };
            self.translate(-delta);

            if let Some((_, resolve)) = first {
                if resolve == normal {
                    self.wall_normal = None;
                }
            } else {
                self.wall_normal = None;
            }
        }
    }


    /// Returns the player's center
    pub fn get_center(&self) -> Vector2 {
        self.collision.average()
    }
}
