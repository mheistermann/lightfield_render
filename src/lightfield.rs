extern crate glium;
extern crate image;
extern crate zip;
extern crate cgmath;
extern crate notify;

use std::io::Read;
use std::fs::File;
use std::io::Cursor;

use glium::backend::Facade;
use glium::texture::{UncompressedUintFormat, MipmapsOption};
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::unsigned_texture2d_array::UnsignedTexture2dArray;


pub struct Lightfield {
    pub tex: UnsignedTexture2dArray,
}

impl Lightfield {
    #![allow(dead_code)]
    pub fn new<F: Facade>(facade: &F, zip_filename: &str) -> Lightfield {
        let zipfile = File::open(zip_filename).unwrap();
        let mut archive = zip::ZipArchive::new(zipfile).unwrap();

        // FIXME n's and sizes
        let nx = 2;
        let ny = 2;
        let width = 1400;
        let height = 800;

        let n = nx*ny;
        let tex = UnsignedTexture2dArray::empty_with_format(
            facade,
            UncompressedUintFormat::U8U8U8U8,
            MipmapsOption::NoMipmap,
            width, height, n).unwrap();

        for i in 0..archive.len()
        {
            let mut file = &mut archive.by_index(i).unwrap();
            let name = String::from(file.name());
            println!("loading {}", name);
            let parts: Vec<&str> = name.split("_").collect();
            let ix:u32 = parts[1].parse().unwrap();
            let iy:u32 = parts[2].parse().unwrap();
            assert!(parts[0] == "out");
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();

            let image = image::load(Cursor::new(contents), image::JPEG).unwrap().to_rgba();
            let image_dimensions = image.dimensions();
            assert!(image_dimensions == (width, height));
            //let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
            let layer = iy*nx+ix;
            println!("loading layer {}", layer);
            let size = width * height * 4;
            let buffer = PixelBuffer::new_empty(facade, size as usize);
            buffer.write(&image.into_raw());
            assert!(layer<n);
            tex.main_level().raw_upload_from_pixel_buffer(buffer.as_slice(),
            0..width,
            0..height, 
            layer..layer+1);
            break; // XXX debug
        }
        Lightfield {tex: tex}
    }
}
