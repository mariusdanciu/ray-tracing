use glam::Vec2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::Instant;

use crate::camera::{Camera, CameraEvent};
use crate::renderer::Renderer;

pub struct App {}

impl App {
    pub fn run(camera: &mut Camera, renderer: &mut Renderer) -> Result<(), String> {
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

        let mut frame_time = Instant::now();
        let mut timer = Instant::now();

        let nanos = 1000000000. / 60.;
        let mut delta: f64 = 0.;
        let mut ups = 0u32;
        let mut fps = 0u32;

        camera.update(CameraEvent::Resize {
            w: size.0 as usize,
            h: size.1 as usize,
        });

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
                        timestamp: _,
                        window_id: _,
                        keycode,
                        scancode: _,
                        keymod: _,
                        repeat: _,
                    } => match keycode {
                        Some(Keycode::W) => {
                            camera.update(CameraEvent::Up);
                            updated = true;
                        }
                        Some(Keycode::S) => {
                            camera.update(CameraEvent::Down);
                            updated = true;
                        }
                        Some(Keycode::A) => {
                            camera.update(CameraEvent::Left);
                            updated = true;
                        }
                        Some(Keycode::D) => {
                            camera.update(CameraEvent::Right);
                            updated = true;
                        }
                        _ => {}
                    },
                    Event::MouseButtonDown {
                        timestamp: _,
                        window_id: _,
                        which: _,
                        mouse_btn: _,
                        clicks: _,
                        x,
                        y,
                    } => {
                        mouse_pressed = true;
                        last_mouse_pos = Vec2::new(x as f32, y as f32);
                        sdl_context.mouse().show_cursor(false);
                    }
                    Event::MouseButtonUp { .. } => {
                        mouse_pressed = false;
                        sdl_context.mouse().show_cursor(true);
                    }

                    Event::MouseMotion {
                        timestamp: _,
                        window_id: _,
                        which: _,
                        mousestate: _,
                        x,
                        y,
                        xrel: _,
                        yrel: _,
                    } => {
                        if mouse_pressed {
                            let mouse_pos = Vec2::new(x as f32, y as f32);

                            let delta = (mouse_pos - last_mouse_pos) * 0.05;

                            last_mouse_pos = mouse_pos;
                            if delta.x != 0.0 || delta.y != 0.0 {
                                camera.update(CameraEvent::RotateXY { delta });
                                updated = true;
                            }
                        }
                    }
                    Event::Window {
                        timestamp: _,
                        window_id: _,
                        win_event,
                    } => match win_event {
                        WindowEvent::SizeChanged(w, h) => {
                            changed = Some((w as usize, h as usize));
                        }
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

            frame_time = Instant::now();

            delta += elapsed / nanos;

            while delta >= 1. {
                // App state updates here.
                ups += 1;
                delta -= 1.;
            }

            if let Some((w, h)) = changed {
                println!("Resized");
                updated = true;
                camera.update(CameraEvent::Resize {
                    w: w as usize,
                    h: h as usize,
                });

                texture = texture_creator
                    .create_texture_streaming(PixelFormatEnum::ABGR8888, w as u32, h as u32)
                    .map_err(|e| e.to_string())?;

                changed = None;
            }

            canvas.clear();
            renderer.render_par(&mut texture, &camera, updated)?;
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
