use std::collections::HashSet;

use glium;
use glium::glutin;

pub use glium::glutin::VirtualKeyCode as KeyCode;
pub use glium::glutin::MouseButton;

pub struct Window {
    events_loop: glutin::EventsLoop,
    display: glium::Display,

    open: bool,

    pressed_keys: HashSet<KeyCode>,
    cursor_position: (u64, u64),
}


impl Window {
    pub fn new(settings: WindowSettings) -> Window {
        // Create a window with event loop
        let events_loop = glutin::EventsLoop::new();

        let (width, height, monitor) = if settings.fullscreen {
            let primary = events_loop.get_primary_monitor();
            let (w, h) = primary.get_dimensions();

            (w, h, Some(primary))
        } else {
            (settings.width, settings.height, None)
        };

        let window = glutin::WindowBuilder::new()
            .with_title(settings.title)
            .with_fullscreen(monitor)
            .with_dimensions(width, height);

        let context = glutin::ContextBuilder::new()
            .with_vsync(settings.vsync)
            .with_srgb(true)
            .with_multisampling(settings.samples);

        let display = glium::Display::new(window, context, &events_loop).unwrap();

        Window {
            events_loop,
            display,

            open: true,

            pressed_keys: HashSet::new(),
            cursor_position: (0, 0),
        }
    }


    /// Return a copy of the display
    pub fn get_display(&self) -> glium::Display {
        self.display.clone()
    }


    /// Poll all events and pass them to a handler
    pub fn handle_events<H: WindowHandler>(&mut self, handler: &mut H) {
        let open = &mut self.open;
        let pressed_keys = &mut self.pressed_keys;
        let cursor_position = &mut self.cursor_position;

        self.events_loop.poll_events(|e| {
            use glium::glutin::{Event, WindowEvent, ElementState};
            match e {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Closed => {
                            *open = false;
                            handler.closed();
                        }

                        WindowEvent::Resized(w, h) => {
                            handler.size_changed(w as u64, h as u64);
                        }

                        WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(key) = input.virtual_keycode {
                                match input.state {
                                    ElementState::Pressed => {
                                        if pressed_keys.insert(key) {
                                            handler.key_pressed(key)
                                        }
                                    }
                                    ElementState::Released => {
                                        pressed_keys.remove(&key);
                                        handler.key_released(key)
                                    }
                                }
                            }
                        }

                        WindowEvent::CursorMoved { position, .. } => {
                            *cursor_position = (position.0.round() as u64, position.1.round() as u64);
                            handler.mouse_moved(cursor_position.0, cursor_position.1);
                        }

                        WindowEvent::MouseInput { button, state, .. } => {
                            match state {
                                ElementState::Pressed => {
                                    handler.mouse_pressed(button, cursor_position.0, cursor_position.1)
                                }
                                ElementState::Released => {
                                    handler.mouse_released(button, cursor_position.0, cursor_position.1)
                                }
                            }
                        }

                        _ => ()
                    }
                }

                _ => ()
            }
        });
    }


    /// Returns true if the window is open, false otherwise
    pub fn is_open(&self) -> bool {
        self.open
    }
}


pub struct WindowSettings {
    pub title: String,

    pub width: u32,
    pub height: u32,

    pub fullscreen: bool,
    pub vsync: bool,

    pub samples: u16,
}


pub trait WindowHandler {
    /// Called when the window is closed
    fn closed(&mut self);

    /// Called when a key is pressed
    fn key_pressed(&mut self, key: KeyCode);

    /// Called when a key is released
    fn key_released(&mut self, key: KeyCode);

    /// Called when a mouse button is pressed
    fn mouse_pressed(&mut self, key: MouseButton, x: u64, y: u64);

    /// Called when a mouse button is released
    fn mouse_released(&mut self, key: MouseButton, x: u64, y: u64);

    /// Called when the cursor has moved
    fn mouse_moved(&mut self, x: u64, y: u64);

    /// Called when the window changes size
    fn size_changed(&mut self, width: u64, height: u64);
}
