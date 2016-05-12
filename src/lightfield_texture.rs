extern crate glium;
extern crate image;
extern crate zip;
extern crate cgmath;
extern crate notify;
extern crate lightfield_loader;

use std::io::Read;
use std::fs::File;
use std::io::Cursor;

use lightfield_loader::{Lightfield, LightfieldView};

use glium::backend::Facade;
use glium::texture::{UncompressedUintFormat, MipmapsOption};
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::unsigned_texture2d_array::UnsignedTexture2dArray;

use image::{DynamicImage, GenericImage, ImageFormat, Pixel, Rgb};


pub struct LightfieldTexture {
    pub tex: UnsignedTexture2dArray,
}

impl LightfieldTexture {
    #![allow(dead_code)]
    pub fn new<F: Facade>(facade: &F, zip_filename: &str) -> LightfieldTexture {
        let lf = Lightfield::from_zip(zip_filename).unwrap();

        // FIXME n's and sizes
        let nx = 1;
        let ny = 1;
        let n = nx*ny;
        let img0 = &lf.views[0].image;
        let width = img0.width();
        let height = img0.height();
        let buffer_size = (width * height * 3) as usize;

        let tex = UnsignedTexture2dArray::empty_with_format(
            facade,
            UncompressedUintFormat::U8U8U8,
            MipmapsOption::NoMipmap,
            width, height, n).unwrap();

        for view in &lf.views {
            let image= &view.image;
            assert!(image.width() == width);
            assert!(image.height() == height);
            match image {
                &DynamicImage::ImageRgb8(_) => {},
                _ => { panic!("Cannot handle this image type"); },
            }
            let layer = 0; // iy*nx+ix;
            println!("loading layer {}", layer);
            assert!(layer<n);
            // TODO: due to wrong pixelbuffer format, texture upload uses format = GL_RED_INTEGER
            // how can i write() to a pixelbuffer with T=(u8,u8,u8,u8) from a [u8] src?
            //let buffer = PixelBuffer::<(u8, u8, u8, u8)>::new_empty(facade, buffer_size);
            let buffer = PixelBuffer::<u8>::new_empty(facade, buffer_size);
            buffer.write(&image.raw_pixels());
            tex.main_level().raw_upload_from_pixel_buffer(buffer.as_slice(),
            0..width,
            0..height,
            layer..layer+1);
            break; // XXX debug
        }
        LightfieldTexture {tex: tex}
    }
}
