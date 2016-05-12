extern crate glium;
extern crate image;
extern crate zip;
extern crate cgmath;
extern crate notify;
extern crate lightfield_loader;

use lightfield_loader::{Lightfield, LightfieldView};

use glium::backend::Facade;
use glium::texture::{MipmapsOption, UncompressedFloatFormat};
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::texture2d_array::Texture2dArray;
use std::iter::FromIterator;

use image::{DynamicImage, GenericImage};


pub struct LightfieldTexture {
    pub tex: Texture2dArray,
}

impl LightfieldTexture {
    #![allow(dead_code)]
    pub fn new<F: Facade>(facade: &F, zip_filename: &str) -> LightfieldTexture {
        let lf = Lightfield::from_zip(zip_filename).unwrap();
        let nlayers = lf.views.len() as u32;
        let img0 = &lf.views[0].image;
        let width = img0.width();
        let height = img0.height();
        let num_pixels = (width * height) as usize;

        let tex = Texture2dArray::empty_with_format(facade,
                                                    UncompressedFloatFormat::U8U8U8,
                                                    MipmapsOption::NoMipmap,
                                                    width,
                                                    height,
                                                    nlayers).unwrap();

        for (_layeridx, view) in lf.views.iter().enumerate() {
            let layeridx = _layeridx as u32;
            let image = &view.image;
            assert!(image.width() == width);
            assert!(image.height() == height);
            match image {
                &DynamicImage::ImageRgb8(_) => {}
                _ => {
                    panic!("Cannot handle this image type");
                }
            }
            debug!("loading layer {}", layeridx);
            let buffer = PixelBuffer::<(u8, u8, u8)>::new_empty(facade, num_pixels);
            let pixels: &Vec<u8> = &image.raw_pixels();
            let rgb_iter = pixels.chunks(3).map(|v| (v[0], v[1], v[2]));
            let tuples: Vec<(u8, u8, u8)> = Vec::from_iter(rgb_iter);
            buffer.write(tuples.as_slice());
            tex.main_level().raw_upload_from_pixel_buffer(buffer.as_slice(),
                                                          0..width,
                                                          0..height,
                                                          layeridx..layeridx + 1);
        }
        unsafe { tex.generate_mipmaps() };
        LightfieldTexture { tex: tex }
    }
}
