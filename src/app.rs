use std::borrow::BorrowMut;
use std::time::{Duration, Instant};

use crate::camera::{self, Camera, CameraEvent};
use fontdue_sdl2::fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue_sdl2::fontdue::Font;
use fontdue_sdl2::FontTexture;
use glam::Vec2;
use rand::{rngs::ThreadRng, Rng, RngCore};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::timer::Timer;
use sdl2::video::{Window, WindowContext};

pub struct App {}

impl App {
    pub fn run<T>(
        state: &mut T,
        renderer: impl Fn(&mut Texture, &Camera, &mut T, f32, bool) -> Result<(), String>,
    ) -> Result<(), String> {
        let sdl_context = sdl2::init()?;

        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Ray Tracing", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();

        let size = canvas.output_size().unwrap();

        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::ABGR8888, size.0, size.1)
            .map_err(|e| e.to_string())?;

        let mut event_pump = sdl_context.event_pump()?;
        let mut changed: Option<(usize, usize)> = None;

        let mut camera = Camera::new();

        let mut frame_time = Instant::now();
        let mut timer = Instant::now();

        let nanos = 1000000000. / 60.;
        let mut delta: f64 = 0.;
        let mut ups = 0u32;
        let mut fps = 0u32;
        let mut time_step = 0.333;

        camera.update(
            CameraEvent::Resize {
                w: size.0 as usize,
                h: size.1 as usize,
            },
            time_step,
        );

        let mut last_mouse_pos = Vec2::new(0., 0.);
        let mut mouse_pressed = false;
        let mut updated = true;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,

                    Event::KeyDown {
                        timestamp,
                        window_id,
                        keycode,
                        scancode,
                        keymod,
                        repeat,
                    } => match keycode {
                        Some(Keycode::Up) => {
                            camera.update(CameraEvent::Up, time_step);
                            updated = true;
                        }
                        Some(Keycode::Down) => {
                            camera.update(CameraEvent::Down, time_step);
                            updated = true;
                        }
                        Some(Keycode::Left) => {
                            camera.update(CameraEvent::Left, time_step);
                            updated = true;
                        }
                        Some(Keycode::Right) => {
                            camera.update(CameraEvent::Right, time_step);
                            updated = true;
                        }
                        _ => {}
                    },
                    Event::MouseButtonDown {
                        timestamp,
                        window_id,
                        which,
                        mouse_btn,
                        clicks,
                        x,
                        y,
                    } => {
                        mouse_pressed = true;
                        last_mouse_pos = Vec2::new(x as f32, y as f32);
                        sdl_context.mouse().show_cursor(false);
                    }
                    Event::MouseButtonUp {
                        timestamp,
                        window_id,
                        which,
                        mouse_btn,
                        clicks,
                        x,
                        y,
                    } => {
                        mouse_pressed = false;
                        sdl_context.mouse().show_cursor(true);
                    }

                    Event::MouseMotion {
                        timestamp,
                        window_id,
                        which,
                        mousestate,
                        x,
                        y,
                        xrel,
                        yrel,
                    } => {
                        if mouse_pressed {
                            let mouse_pos = Vec2::new(x as f32, y as f32);

                            let delta = (mouse_pos - last_mouse_pos) * 0.05;

                            last_mouse_pos = mouse_pos;
                            if delta.x != 0.0 || delta.y != 0.0 {
                                camera.update(CameraEvent::RotateXY { delta }, time_step);
                                updated = true;
                            }
                        }
                        //println!("{:?} || {}, {}", mousestate, x, y);
                    }
                    Event::Window {
                        timestamp,
                        window_id,
                        win_event,
                    } => match win_event {
                        WindowEvent::Resized(w, h) => {
                            changed = Some((w as usize, h as usize));
                        }
                        WindowEvent::Exposed => {}
                        _ => {}
                    },
                    _ => {}
                }
            }

            let elapsed = frame_time.elapsed().as_nanos() as f64;
            time_step = elapsed.min(0.0333f64) as f32;

            frame_time = Instant::now();

            delta += elapsed / nanos;

            while delta >= 1. {
                // App state updates here.
                ups += 1;
                delta -= 1.;
            }

            if let Some((w, h)) = changed {
                println!("Resized");
                camera.update(
                    CameraEvent::Resize {
                        w: w as usize,
                        h: h as usize,
                    },
                    time_step,
                );

                texture = texture_creator
                    .create_texture_streaming(PixelFormatEnum::ABGR8888, w as u32, h as u32)
                    .map_err(|e| e.to_string())?;

                changed = None;
            }

            canvas.clear();
            renderer(&mut texture, &camera, state, time_step, updated)?;
            canvas.copy(&texture, None, None)?;
            canvas.present();

            updated = false;
            fps += 1;

            let millis = timer.elapsed().as_millis();

            if millis > 1000 {
                timer = Instant::now();
                canvas
                    .window_mut()
                    .set_title(format!("ups {} / fps {}", ups, fps).as_str())
                    .map_err(|e| e.to_string())?;
                ups = 0;
                fps = 0;
            }
        }

        Ok(())
    }
}