use std::{sync::Arc, time::Instant};

use glam::Vec4;
use rand::rngs::ThreadRng;
use sdl2::{render::Texture, timer::Timer};

use crate::{camera::Camera, ray::Ray, scene::Scene};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug, Copy, Clone)]
struct Chunk {
    size: usize,
    pixel_offset: usize,
}

pub struct Renderer {
    pub scene: Arc<Scene>,
    pub accumulated: Vec<Vec4>,
    pub frame_index: u32,
}

impl Renderer {
    pub fn new(scene: Scene) -> Renderer {
        Renderer {
            scene: Arc::new(scene),
            accumulated: vec![],
            frame_index: 1,
        }
    }
    fn render_chunk(
        &mut self,
        camera: &Camera,
        rnd: &mut ThreadRng,
        chunk: Chunk,
        bytes: &mut [u8],
        start_time: Instant
    ) {
        let mut i = 0;

        for pos in 0..chunk.size {
            let ray_dir = camera.ray_directions[pos + chunk.pixel_offset];

            self.accumulated[pos] += self.scene.pixel(
                Ray {
                    origin: camera.position,
                    direction: ray_dir,
                },
                rnd,
                start_time
            );

            let mut accumulated = self.accumulated[pos];
            accumulated /= self.frame_index as f32;
            accumulated = accumulated.clamp(Vec4::ZERO, Vec4::ONE);

            let color = Scene::to_rgba(accumulated);

            bytes[i] = color.0;
            bytes[i + 1] = color.1;
            bytes[i + 2] = color.2;
            bytes[i + 3] = color.3;

            i += 4;
        }
    }

    pub fn render_par(
        &mut self,
        texture: &mut Texture,
        img: &mut Vec<u8>,
        camera: &Camera,
        updated: bool,
        num_chunks: usize,
        start_time: Instant
    ) -> Result<(), String> {
        let w = camera.width;
        let h = camera.height;

        if updated {
            self.accumulated = vec![Vec4::ZERO; w * h];
            self.frame_index = 1;
        }

        if self.frame_index > self.scene.max_frames_rendering {
            return Ok(());
        }

        let img_len = img.len();
        let img_chunk_size = (img_len / (num_chunks * 4)) * 4;

        let chunks: Vec<(usize, &mut [u8])> = img.chunks_mut(img_chunk_size).enumerate().collect();

        let col: Vec<Renderer> = chunks
            .into_par_iter()
            .map(|e| {
                let mut rnd = rand::thread_rng();
                let buf_len = e.1.len();

                let acc_size = buf_len / 4;

                let offset = e.0 * acc_size;

                let mut acc = vec![Vec4::ZERO; acc_size];
                acc.copy_from_slice(&self.accumulated[offset..(offset + acc_size)]);

                let mut s = Renderer {
                    scene: self.scene.clone(),
                    accumulated: acc,
                    frame_index: self.frame_index,
                };

                let chunk = Chunk {
                    size: acc_size,
                    pixel_offset: offset,
                };

                s.render_chunk(camera, &mut rnd, chunk, e.1, start_time);
                s
            })
            .collect();

        let mut offset = 0;
        for c in col {
            let len = c.accumulated.len();
            self.accumulated[offset..offset + len].copy_from_slice(c.accumulated.as_slice());
            offset += len;
        }

        texture
            .update(None, img.as_slice(), w * 4)
            .map_err(|e| e.to_string())?;

        self.frame_index += 1;

        Ok(())
    }
}
