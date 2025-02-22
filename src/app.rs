use glam::Vec2;
use sdl2::event::{Event, WindowEvent};

use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::Instant;

use crate::camera::{Camera, CameraEvent};

use crate::renderer::Renderer;
use crate::scene::{self, Scene};
use crate::utils::errors::AppError;

pub struct App {}

impl App {
    pub fn run(
        camera: &mut Camera,
        renderer: &mut Renderer,
        scene: &mut Scene,
    ) -> Result<(), AppError> {
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

        let nanos = 1000000000. / 80.;
        let mut delta: f64 = 0.;
        let mut ups = 0u32;
        let mut fps = 0u32;

        camera.update(
            &vec![CameraEvent::Resize {
                w: size.0 as usize,
                h: size.1 as usize,
            }],
            frame_time.elapsed().as_millis() as f32 / 1000.,
        );

        let mut img: Vec<u8> = vec![0; (size.0 * size.1 * 4) as usize];

        let mut last_mouse_pos = Vec2::new(0., 0.);
        let mut mouse_pressed = false;
        let mut updated = true;

        let mut up = false;
        let mut down = false;
        let mut left = false;
        let mut right = false;
        let num_cores = 30; //num_cpus::get();

        'running: loop {
            let elapsed = frame_time.elapsed();
            let elapsed_nanos = elapsed.as_nanos() as f64;
            let ts = elapsed.as_secs_f32();
            let mut rotateXY: Option<Vec2> = None;

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
                        keycode: Some(code),
                        scancode: _,
                        keymod: _,
                        repeat: false,
                    } => {
                        match code {
                            Keycode::W => up = true,
                            Keycode::S => down = true,
                            Keycode::A => left = true,
                            Keycode::D => right = true,
                            _ => {}
                        };
                    }

                    Event::KeyUp {
                        timestamp,
                        window_id,
                        keycode: Some(code),
                        scancode,
                        keymod,
                        repeat,
                    } => {
                        match code {
                            Keycode::W => up = false,
                            Keycode::S => down = false,
                            Keycode::A => left = false,
                            Keycode::D => right = false,
                            _ => {}
                        };
                    }

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
                        rotateXY = None;
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
                                rotateXY = Some(delta);
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
                            img = vec![0; (w * h * 4) as usize];
                        }
                        WindowEvent::Resized(w, h) => {
                            changed = Some((w as usize, h as usize));
                            img = vec![0; (w * h * 4) as usize];
                        }
                        WindowEvent::Exposed => {}
                        _ => {}
                    },
                    _ => {}
                }
            }

            frame_time = Instant::now();

            delta += elapsed_nanos / nanos;
            while delta >= 1. {
                let mut events: Vec<CameraEvent> = vec![];
                if up {
                    events.push(CameraEvent::Up)
                }
                if down {
                    events.push(CameraEvent::Down)
                }
                if left {
                    events.push(CameraEvent::Left)
                }
                if right {
                    events.push(CameraEvent::Right)
                }
                if let Some(delta) = rotateXY {
                    events.push(CameraEvent::RotateXY { delta })
                }

                if !events.is_empty() {
                    camera.update(&events, ts);
                    updated = true;
                }

                // App state updates here.
                if let Some(f) = scene.update_func {
                    let u = f(scene, ts);
                    if !updated {
                        updated = u;
                    }
                }

                ups += 1;
                delta -= 1.;
            }

            if let Some((w, h)) = changed {
                updated = true;
                camera.update(
                    &vec![CameraEvent::Resize {
                        w: w as usize,
                        h: h as usize,
                    }],
                    ts,
                );

                texture = texture_creator
                    .create_texture_streaming(PixelFormatEnum::ABGR8888, w as u32, h as u32)
                    .map_err(|e| e.to_string())?;

                changed = None;
            }

            canvas.clear();
            renderer.render_par(
                scene,
                &mut texture,
                &mut img,
                &camera,
                updated,
                num_cores,
                ts,
            )?;
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
