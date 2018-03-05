use trap::Vector2;
use rax::collision::*;

use rax::Renderer;

pub struct Player {
    collision: ConvexHull,
    rag_doll: RagDoll,

    center: Vector2,
    velocity: Vector2,

    move_direction: Option<MoveDirection>,
    face_direction: MoveDirection,

    ground_normal: Option<Vector2>,
    wall_normal: Option<Vector2>,

    jumping: bool,
    sliding: bool,

    commands: Vec<PlayerCommand>,
}


#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PlayerCommand {
    MoveLeft,
    MoveRight,

    Jump,
    StopJump,

    Drop,
    Slide
}


#[derive(Eq, PartialEq, Copy, Clone)]
enum MoveDirection {
    Left,
    Right,
}


/// Stores the positions of all limb endpoints
struct RagDoll {
    shoulder: Vector2,
    arm_joints: [Vector2; 2],
    hands: [Vector2; 2],

    hip: Vector2,
    leg_joints: [Vector2; 2],
    feet: [Vector2; 2],
}


impl Player {
    pub fn new(position: Vector2) -> Player {
        let w = 24.0;
        let h = 45.0;
        let mut player = Player {
            //collision: Circle::new(position, 45.0 / 2.0),
            collision: ConvexHull::from_points(&[
                Vector2 { x: position.x - w / 2.0, y: position.y - h / 2.0 },
                Vector2 { x: position.x + w / 2.0, y: position.y - h / 2.0 },
                Vector2 { x: position.x + w / 2.0, y: position.y + h / 2.0 },
                Vector2 { x: position.x - w / 2.0, y: position.y + h / 2.0 },
            ]),
            rag_doll: RagDoll {
                shoulder: Vector2::new(0.0, 0.0),
                arm_joints: [Vector2::new(0.0, 0.0); 2],
                hands: [Vector2::new(0.0, 0.0); 2],
                hip: Vector2::new(0.0, 0.0),
                leg_joints: [Vector2::new(0.0, 0.0); 2],
                feet: [Vector2::new(0.0, 0.0); 2],
            },

            center: position,
            velocity: Vector2 { x: 0.0, y: 0.0 },

            move_direction: None,
            face_direction: MoveDirection::Right,

            ground_normal: None,
            wall_normal: None,

            jumping: false,
            sliding: false,

            commands: Vec::new(),
        };

        player.rag_doll = player.get_rag_doll();

        player
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
                    if self.sliding {
                        self.velocity.y -= self.velocity.y * dt * 2.0;
                    } else {
                        self.velocity.y -= self.velocity.y * dt * 9.0;
                    }
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

        let rag_doll = self.get_rag_doll();
        let factor = 15.0;


        self.rag_doll.hip += factor * dt * (rag_doll.hip - self.rag_doll.hip);

        // Keep the hip in the center
        let delta = self.center - self.rag_doll.hip;

        self.rag_doll.hip += delta;
        self.rag_doll.shoulder += factor * dt * (rag_doll.shoulder - self.rag_doll.shoulder) + delta;

        for i in 0..2 {
            self.rag_doll.arm_joints[i] += factor * dt * (rag_doll.arm_joints[i] - self.rag_doll.arm_joints[i]) + delta;
            self.rag_doll.hands[i] += factor * dt * (rag_doll.hands[i] - self.rag_doll.hands[i]) + delta;
            self.rag_doll.leg_joints[i] += factor * dt * (rag_doll.leg_joints[i] - self.rag_doll.leg_joints[i]) + delta;
            self.rag_doll.feet[i] += factor * dt * (rag_doll.feet[i] - self.rag_doll.feet[i]) + delta;
        }
    }


    /// Draw the player
    pub fn draw(&self, renderer: &mut Renderer) {
        {
            renderer.color = [0.0, 0.0, 1.0, 0.3];
            // renderer.fill_convex(self.collision.get_points());

            renderer.color = [1.0, 1.0, 1.0, 1.0];
            renderer.line_width = 2.0;

            let rag_doll = &self.rag_doll;

            renderer.draw_rounded_line(rag_doll.shoulder, rag_doll.hip);
            for i in 0..2 {
                renderer.draw_rounded_line(rag_doll.shoulder, rag_doll.arm_joints[i]);
                renderer.draw_rounded_line(rag_doll.arm_joints[i], rag_doll.hands[i]);

                renderer.draw_rounded_line(rag_doll.hip, rag_doll.leg_joints[i]);
                renderer.draw_rounded_line(rag_doll.leg_joints[i], rag_doll.feet[i]);
            }
        }
    }


    /// Returns the current rag doll
    fn get_rag_doll(&self) -> RagDoll {
        let mut rag_doll = RagDoll {
            shoulder: Vector2::new(0.0, 0.0),
            arm_joints: [Vector2::new(0.0, 0.0); 2],
            hands: [Vector2::new(0.0, 0.0); 2],
            hip: Vector2::new(0.0, 0.0),
            leg_joints: [Vector2::new(0.0, 0.0); 2],
            feet: [Vector2::new(0.0, 0.0); 2],
        };

        let bounds = self.collision.bounding_box();
        let left = bounds.left;
        let right = bounds.right;
        let top = bounds.top;
        let bottom = bounds.bottom;

        let mid = Vector2::new((left + right) / 2.0, (top + bottom) / 2.0);
        let size = Vector2::new(right - left, bottom - top);

        // Construct unit vector with an angle between x axis
        let angle = |angle: f64| {
            use std::f64::consts::PI;
            let rad = angle / 180.0 * PI;
            Vector2::new(rad.cos(), -rad.sin())
        };

        // Map value in range [0, 1] to range [min, max]
        let map = |v: f64, min: f64, max: f64| { v * (max - min) + min };

        let fmod = |a: f64, b: f64| { a - (a / b).floor() * b };

        if let Some(normal) = self.wall_normal {
            let angle = |a: f64| {
                angle(if normal.x > 0.0 { a } else { 180.0 - a })
            };

            rag_doll.hip.x = mid.x;
            rag_doll.hip.y = mid.y;

            rag_doll.shoulder = rag_doll.hip + size.y / 3.0 * angle(75.0);

            rag_doll.arm_joints = [
                rag_doll.shoulder + size.y / 5.0 * angle(160.0),
                rag_doll.shoulder + size.y / 5.0 * angle(210.0)
            ];
            rag_doll.hands = [
                rag_doll.arm_joints[0] + size.y / 5.0 * angle(140.0),
                rag_doll.arm_joints[1] + size.y / 5.0 * angle(140.0)
            ];

            rag_doll.leg_joints = [
                rag_doll.hip + size.y / 4.0 * angle(130.0),
                rag_doll.hip + size.y / 4.0 * angle(240.0),
            ];

            rag_doll.feet = [
                rag_doll.leg_joints[0] + size.y / 4.0 * angle(240.0),
                rag_doll.leg_joints[1] + size.y / 4.0 * angle(240.0),
            ];

            // Climbing wall
        } else if let Some(normal) = self.ground_normal {
            let angle = |a: f64| {
                angle(if self.velocity.x > 0.0 { a } else { 180.0 - a })
            };

            let mut p = fmod(mid.x, size.x * 4.0) / (size.x * 4.0) * 2.0;

            // Running
            rag_doll.hip.x = mid.x;
            rag_doll.hip.y = mid.y;

            let swap = if p > 1.0 {
                p -= 1.0;
                true
            } else {
                false
            };

            rag_doll.shoulder = rag_doll.hip + size.y / 3.0 * angle(map(p, 80.0, 80.0));

            let arm_angles = [
                map(p, 360.0 - 30.0, 360.0 - 160.0),
                map(p, 360.0 - 160.0, 360.0 - 30.0)
            ];

            rag_doll.arm_joints = [
                rag_doll.shoulder + size.y / 5.0 * angle(arm_angles[0]),
                rag_doll.shoulder + size.y / 5.0 * angle(arm_angles[1])
            ];
            rag_doll.hands = [
                rag_doll.arm_joints[0] + size.y / 5.0 * angle(arm_angles[0] + 70.0),
                rag_doll.arm_joints[1] + size.y / 5.0 * angle(arm_angles[1] + 70.0)
            ];

            let leg_angles = [
                map(p, 360.0 - 60.0, 360.0 - 140.0),
                map(p, 360.0 - 140.0, 360.0 - 60.0)
            ];

            rag_doll.leg_joints = [
                rag_doll.hip + size.y / 4.0 * angle(leg_angles[0]),
                rag_doll.hip + size.y / 4.0 * angle(leg_angles[1]),
            ];

            rag_doll.feet = [
                rag_doll.leg_joints[0] + size.y / 4.0 * angle(leg_angles[0] - map(p, 0.0, 60.0)),
                rag_doll.leg_joints[1] + size.y / 4.0 * angle(leg_angles[1] - map(p, 60.0, 0.0)),
            ];

            if swap {
                rag_doll.arm_joints.swap(0, 1);
                rag_doll.hands.swap(0, 1);
                rag_doll.leg_joints.swap(0, 1);
                rag_doll.feet.swap(0, 1);
            }
        } else {
            let angle = |a: f64| {
                angle(if self.velocity.x > 0.0 { a } else { 180.0 - a })
            };

            // Falling/Jumping
            let vy = {
                let max = -50.0;
                let min = 75.0;
                let v = (self.velocity.y - min) / (max - min);
                if v > 1.0 { 1.0 } else if v < 0.0 { 0.0 } else { v }
            };

            rag_doll.hip.x = mid.x;
            rag_doll.hip.y = mid.y;

            rag_doll.shoulder = rag_doll.hip + size.y / 3.0 * angle(map(vy, 100.00, 70.0));

            rag_doll.arm_joints = [
                rag_doll.shoulder + size.y / 5.0 * angle(map(vy, 0.0, -60.0)),
                rag_doll.shoulder + size.y / 5.0 * angle(map(vy, 180.0, 210.0))
            ];
            rag_doll.hands = [
                rag_doll.arm_joints[0] + size.y / 5.0 * angle(map(vy, 30.0, 50.0)),
                rag_doll.arm_joints[1] + size.y / 5.0 * angle(map(vy, 200.0, 220.0))
            ];

            rag_doll.leg_joints = [
                rag_doll.hip + size.y / 4.0 * angle(map(vy, -60.0, 10.0)),
                rag_doll.hip + size.y / 4.0 * angle(map(vy, 360.0 - 10.0, 240.0)),
            ];

            rag_doll.feet = [
                rag_doll.leg_joints[0] + size.y / 4.0 * angle(map(vy, 360.0 - 60.0, 240.0)),
                rag_doll.leg_joints[1] + size.y / 4.0 * angle(map(vy, 220.0, 240.0)),
            ];
        };


        rag_doll
    }


    /// Make the player execute a command
    pub fn submit_command(&mut self, command: PlayerCommand) {
        self.commands.push(command);
    }


    /// Translate the player
    pub fn translate(&mut self, amount: Vector2) {
        self.collision.translate(amount);
        self.center += amount;
    }


    /// Handle any commands
    fn handle_commands(&mut self, dt: f64) {
        self.move_direction = None;

        self.sliding = false;

        let commands = self.commands.clone();
        for command in commands {
            match command {
                PlayerCommand::MoveLeft => { self.move_direction(MoveDirection::Left, dt); }
                PlayerCommand::MoveRight => { self.move_direction(MoveDirection::Right, dt); }

                PlayerCommand::Jump => { self.jumping = true; }
                PlayerCommand::StopJump => { self.jumping = false; }

                PlayerCommand::Drop => { self.wall_normal = None; }
                PlayerCommand::Slide => {self.sliding = true; }
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
                self.face_direction = direction;
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
                if i > 10 {
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
        if resolve.x != 0.0 {
            println!("Resolve: {:?}", resolve);
        }

        self.translate(-resolve);

        let normal = -resolve.norm();

        // Slide
        if normal.dot(self.velocity) < 0.0 {
            let plane = Vector2::new(normal.y, -normal.x);

            let v_dot = normal.dot(-self.velocity.norm());
            let dot = plane.dot(self.velocity.norm());

            self.velocity = if v_dot <= (50.0 as f64).cos() && self.move_direction.is_some() {
                plane * self.velocity.len() * if dot > 0.0 { dot.powf(1.0 / 4.0) } else { -(-dot).powf(1.0 / 4.0) }
            } else {
                plane * plane.dot(self.velocity)
            }
        }


        // Check if grounded or wall sliding
        if normal.dot(Vector2::new(0.0, -1.0)) > 0.5 {
            self.ground_normal = Some(normal);
            self.wall_normal = None;
        } else {
            let dot = normal.dot(Vector2::new(1.0, 0.0));

            if dot > 0.95 || dot < -0.95 {
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
        self.center
    }
}
