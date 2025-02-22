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
    pub accumulated: Vec<Vec4>,
    pub frame_index: u32,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            accumulated: vec![],
            frame_index: 1,
        }
    }
    fn render_chunk(
        &mut self,
        scene: &Scene,
        camera: &Camera,
        rnd: &mut ThreadRng,
        chunk: Chunk,
        bytes: &mut [u8],
        time: f32,
    ) {
        let mut i = 0;

        for pos in 0..chunk.size {
            let ray_dir = camera.ray_directions[pos + chunk.pixel_offset];

            let color = if scene.enable_accumulation {
                //println!("accumulate {}", self.frame_index);
                self.accumulated[pos] += scene.pixel(
                    Ray {
                        origin: camera.position,
                        direction: ray_dir,
                    },
                    rnd,
                    time,
                );

                let mut accumulated = self.accumulated[pos];
                accumulated /= self.frame_index as f32;
                accumulated = accumulated.clamp(Vec4::ZERO, Vec4::ONE);

                Scene::to_rgba(accumulated)
            } else {
                let c = scene.pixel(
                    Ray {
                        origin: camera.position,
                        direction: ray_dir,
                    },
                    rnd,
                    time,
                );
                self.accumulated[pos] = c.clamp(Vec4::ZERO, Vec4::ONE);
                Scene::to_rgba(self.accumulated[pos])
            };

            bytes[i] = color.0;
            bytes[i + 1] = color.1;
            bytes[i + 2] = color.2;
            bytes[i + 3] = color.3;

            i += 4;
        }
    }

    pub fn render_par(
        &mut self,
        scene: &Scene,
        texture: &mut Texture,
        img: &mut Vec<u8>,
        camera: &Camera,
        updated: bool,
        num_chunks: usize,
        time: f32,
    ) -> Result<(), String> {
        let w = camera.width;
        let h = camera.height;

        if updated {
            self.accumulated = vec![Vec4::ZERO; w * h];
            self.frame_index = 1;
        }

        if self.frame_index > scene.max_frames_rendering
            || (self.frame_index > 1 && !scene.enable_accumulation && !scene.diffuse)
        {
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

                let k = &self.accumulated[offset..(offset + acc_size)];

                let mut s = Renderer {
                    accumulated: k.to_vec(),
                    frame_index: self.frame_index,
                };

                let chunk = Chunk {
                    size: acc_size,
                    pixel_offset: offset,
                };

                s.render_chunk(scene, camera, &mut rnd, chunk, e.1, time);
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
