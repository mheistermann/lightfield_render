extern crate glium;
extern crate image;
extern crate zip;
extern crate cgmath;
extern crate notify;
extern crate lightfield_loader;

use std::io::Read;
use std::fs::File;
use std::io::Cursor;
use std::mem;

use lightfield_loader::{Lightfield, LightfieldView};

use glium::backend::Facade;
use glium::texture::{MipmapsOption, UncompressedFloatFormat};
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::texture2d_array::Texture2dArray;
use std::iter::FromIterator;

use image::{DynamicImage, GenericImage, ImageFormat, Pixel, Rgb};


pub struct LightfieldTexture {
    pub tex: Texture2dArray,
}

impl LightfieldTexture {
    #![allow(dead_code)]
    pub fn new<F: Facade>(facade: &F, zip_filename: &str) -> LightfieldTexture {
        let lf = Lightfield::from_zip(zip_filename).unwrap();

        // FIXME n's and sizes
        let nx = 1;
        let ny = 1;
        let n = nx * ny;
        let img0 = &lf.views[0].image;
        let width = img0.width();
        let height = img0.height();
        let num_pixels = (width * height) as usize;

        let tex = Texture2dArray::empty_with_format(facade,
                                                    UncompressedFloatFormat::U8U8U8,
                                                    MipmapsOption::NoMipmap,
                                                    width,
                                                    height,
                                                    n)
                      .unwrap();

        for view in &lf.views {
            let image = &view.image;
            assert!(image.width() == width);
            assert!(image.height() == height);
            match image {
                &DynamicImage::ImageRgb8(_) => {}
                _ => {
                    panic!("Cannot handle this image type");
                }
            }
            let layer = 0; // iy*nx+ix;
            println!("loading layer {}", layer);
            assert!(layer < n);
            // TODO: due to wrong pixelbuffer format, texture upload uses format = GL_RED_INTEGER
            // how can i write() to a pixelbuffer with T=(u8,u8,u8,u8) from a [u8] src?
            let buffer = PixelBuffer::<(u8, u8, u8)>::new_empty(facade, num_pixels);
            let pixels: &Vec<u8> = &image.raw_pixels();

            /* //disabled code: format-changing copy for now to make sure no bugs are here
            // better make sure the rust tuple representation is packed,
            // so we can kind of safely transmute
            assert!(mem::size_of::<[(u8,u8,u8); 2]>() == 6);
            let pixel_tuples = unsafe {mem::transmute::<&[u8], &[(u8, u8, u8)]>(pixels)};
            buffer.write(pixel_tuples);
            */
            let rgb_iter = pixels.chunks(3).map(|v| (v[0], v[1], v[2]));
            let tuples: Vec<(u8,u8,u8)> = Vec::from_iter(rgb_iter);
            println!("pixels: {}", pixels.len());
            println!("tuples: {}", tuples.len());
            buffer.write(tuples.as_slice());
            tex.main_level().raw_upload_from_pixel_buffer(buffer.as_slice(),
                                                          0..width,
                                                          0..height,
                                                          layer..layer + 1);
            break; // XXX debug
        }
        unsafe { tex.generate_mipmaps() };
        LightfieldTexture { tex: tex }
    }
}
