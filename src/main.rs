#[macro_use]
extern crate glium;
extern crate image;
extern crate zip;
extern crate cgmath;
extern crate notify;

use cgmath::{Vector2, Vector3, Vector4};
use cgmath::{Basis3};
use cgmath::{Matrix, SquareMatrix, Matrix3, Matrix4};
use cgmath::{Angle, Rad, Deg};
use cgmath::{Rotation, Rotation3};
use cgmath::{PerspectiveFov};

use glium::glutin::{Event, MouseButton, ElementState, MouseScrollDelta};
use glium::debug::DebugCallbackBehavior;

//use glium::texture::{Texture2d, RawImage2d};
use glium::uniforms::Sampler;
//use glium::buffer::{Buffer, BufferMode, BufferSlice};

mod lightfield;
use lightfield::Lightfield;

mod reloading_program;
use reloading_program::ReloadingProgram;



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

    let mut reloading_program = ReloadingProgram::from_source(&display, "shaders/lightfield.vsh", "shaders/lightfield.fsh", None);

    let lf = Lightfield::new(&display, "chess.jpg.zip");
    let lf_sampler = Sampler::new(&lf.tex);

    let mut cursor_pos          = Vector2::new(0f32, 0f32);
    let mut cursor_startpos     = Vector2::new(0f32, 0f32);
    let mut cursor_totaloff     = Vector2::new(0f32, 0f32);
    let mut cursor_totaloff_old = Vector2::new(0f32, 0f32);
    let mut scale = 1f32;
    let mut mouse_pressed = false;
    let mut mouse_scrolled = false;
    let mut mat_model = Matrix4::<f32>::identity();
    let mut mat_view = Matrix4::<f32>::identity();
    mat_view[3][2] = -5.0;
    let mut window_size = display.get_window().unwrap().get_inner_size_pixels().unwrap();
    let mut shaders_broken = false;
    loop {
        if shaders_broken {
            println!("Shaders are broken, waiting for updated shader before resuming...");
            reloading_program.wait_for_change();
        }
        let program = match reloading_program.current() {
            &Err(ref err) => {
                println!("Error compiling shader: {:?}.", err);
                shaders_broken = true;
                continue
            }
            &Ok(ref prog) => {
                shaders_broken = false;
                prog
            }
        };

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::Resized(w,h) => {
                    window_size = (w,h);
                },
                Event::MouseMoved(x, y) => {
                    cursor_pos.x = x as f32 / window_size.0 as f32 - 0.5;
                    cursor_pos.y = y as f32 / window_size.1 as f32 - 0.5;
                },
                Event::MouseInput(ElementState::Pressed,  MouseButton::Left) => {
                    cursor_startpos = cursor_pos;
                    mouse_pressed = true;
                },
                Event::MouseInput(ElementState::Released, MouseButton::Left) => {
                    cursor_totaloff_old = cursor_totaloff;
                    mouse_pressed = false;
                },
                Event::MouseWheel(MouseScrollDelta::LineDelta(_, lines), _) => {
                    scale *= f32::powf(1.1, lines);
                    //println!("scroll: {}, scale {}", lines, scale);
                    mouse_scrolled = true;
                },
                _ => ()
            }
        }
        if mouse_pressed || mouse_scrolled {
            let offset = Vector2::from((cursor_startpos - cursor_pos));
            cursor_totaloff = cursor_totaloff_old + offset;
            //println!("{:?}", cursor_totaloff);
            let yaw   = Rad::<f32>::full_turn() * cursor_totaloff.x;
            let pitch   = Rad::<f32>::full_turn() * cursor_totaloff.y;
            let mat_rot = Matrix4::<f32>::from(Matrix3::from_euler(pitch, yaw, Rad::zero()));
            let mat_scale = Matrix4::<f32>::from_diagonal(Vector4::new(scale, scale, scale, 1.0));
            mat_model = mat_rot * mat_scale;
            mouse_scrolled = false; // only recompute once!
        }

        let mut target = display.draw();
        target.clear_color(0.3, 0.3, 0.3, 1.0);
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
