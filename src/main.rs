#[macro_use]
extern crate glium;
extern crate image;
extern crate zip;
extern crate cgmath;
extern crate notify;


use std::io::Read;
use std::fs::File;
use std::io::Cursor;

use notify::{RecommendedWatcher, Error, Watcher};
use std::sync::mpsc::channel;

use cgmath::{Vector, Vector2, Vector3};
use cgmath::{Basis3};
use cgmath::{Matrix, SquareMatrix, Matrix3, Matrix4};
use cgmath::{Angle, Rad, Deg};
use cgmath::{Rotation, Rotation3};
use cgmath::{PerspectiveFov};

use glium::glutin::{Event, MouseButton, ElementState};
use glium::debug::DebugCallbackBehavior;

use glium::backend::Facade;
//use glium::texture::{Texture2d, RawImage2d};
use glium::texture::{UncompressedUintFormat, MipmapsOption};
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::unsigned_texture2d_array::UnsignedTexture2dArray;
use glium::uniforms::Sampler;
//use glium::buffer::{Buffer, BufferMode, BufferSlice};

struct Lightfield {
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

fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .build_glium_debug(DebugCallbackBehavior::PrintAll)
        .unwrap();



    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
    }

    implement_vertex!(Vertex, position, tex_coords);

    let vertex1 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] };
    let vertex2 = Vertex { position: [ 0.0,  0.5], tex_coords: [0.0, 1.0] };
    let vertex3 = Vertex { position: [ 0.5, -0.25], tex_coords: [1.0, 0.0] };
    let shape = vec![vertex1, vertex2, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 330

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 model;
        uniform mat4 view;
        uniform mat4 projection;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = projection * view * model * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 330

        in vec2 v_tex_coords;
        out vec4 color;

        uniform usampler2DArray tex;

        void main() {
            color = texture(tex, vec3(v_tex_coords, 1));
            //color = vec4(1,0,0,1);
        }
    "#;

    let res = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None);
    let program = match res {
        Err(err) => {println!("{}", err); None}
        Ok(prog) => Some(prog)
    }.unwrap();

    let lf = Lightfield::new(&display, "chess.jpg.zip");
    let lf_sampler = Sampler::new(&lf.tex);

    let mut cursor_pos          = Vector2::new(0f32, 0f32);
    let mut cursor_startpos     = Vector2::new(0f32, 0f32);
    let mut cursor_totaloff     = Vector2::new(0f32, 0f32);
    let mut cursor_totaloff_old = Vector2::new(0f32, 0f32);
    let mut mouse_pressed = false;
    let mut mat_model = Matrix4::<f32>::identity();
    let mut mat_view = Matrix4::<f32>::identity();
    mat_view[3][2] = -5.0;
    let mut window_size = display.get_window().unwrap().get_inner_size_pixels().unwrap();
    loop {
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::Resized(w,h) => {
                    window_size = (w,h);
                },
                Event::MouseMoved(m) => {
                    cursor_pos.x = m.0 as f32 / window_size.0 as f32 - 0.5;
                    cursor_pos.y = m.1 as f32 / window_size.1 as f32 - 0.5;
                },
                Event::MouseInput(ElementState::Pressed,  MouseButton::Left) => {
                    cursor_startpos = cursor_pos;
                    mouse_pressed = true;
                },
                Event::MouseInput(ElementState::Released, MouseButton::Left) => {
                    cursor_totaloff_old = cursor_totaloff;
                    mouse_pressed = false;
                },
                _ => ()
            }
        }
        if mouse_pressed {
            let offset = Vector2::from((cursor_startpos - cursor_pos));
            cursor_totaloff = cursor_totaloff_old + offset;
            //println!("{:?}", cursor_totaloff);
            let yaw   = Rad::<f32>::full_turn() * cursor_totaloff.x;
            let pitch   = Rad::<f32>::full_turn() * cursor_totaloff.y;
            mat_model = Matrix4::<f32>::from(Matrix3::from_euler(pitch, yaw, Rad::zero()));
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        let (width, height) = target.get_dimensions();

        let mat_projection: Matrix4<f32> =  PerspectiveFov {
            fovy:   Deg::new(60f32).into(),
            aspect: height as f32 / width as f32,
            near:   0.1,
            far:    1024.,
        }.into();

        let uniforms = uniform! {
            model:      Into::<[[f32; 4]; 4]>::into(mat_model),
            view:       Into::<[[f32; 4]; 4]>::into(mat_view),
            projection: Into::<[[f32; 4]; 4]>::into(mat_projection),
            tex: lf_sampler,
        };


        target.draw(&vertex_buffer, &indices, &program, &uniforms,
                    &Default::default()).unwrap();

        target.finish().unwrap();
        /*
        println!("model: {:#?}", mat_model);
        println!("view: {:#?}", mat_view);
        println!("projection: {:#?}", mat_projection);
        println!("");
        */
    }
}
