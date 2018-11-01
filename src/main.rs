extern crate gl;
extern crate glutin;
extern crate nalgebra;

use glutin::dpi::*;
use glutin::GlContext;
use nalgebra::{Matrix4, Vector3, Point3, Perspective3, Isometry3};

use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;
use std::time;

#[derive(PartialEq, Debug)]
enum MoveVertical {
    Up,
    Down
}

#[derive(PartialEq, Debug)]
enum MoveHorizontal {
    Left,
    Right
}

#[derive(Debug)]
struct Camera {
    position: Point3<f32>,
    target: Option<Point3<f32>>,
    direction: Vector3<f32>,
    up: Vector3<f32>,
    projection: Perspective3<f32>,

    horizontal: Option<MoveHorizontal>,
    vertical: Option<MoveVertical>
}

impl Camera {
    fn new(aspect: f32, fov: f32, z_near: f32, z_far: f32) -> Camera {
        Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            target: Some(Point3::new(0.0, 0.0, 10.0)),
            direction: Vector3::new(0.0, 0.0, 1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            projection: Perspective3::new(aspect,
                                          fov,
                                          z_near,
                                          z_far),
            horizontal: None,
            vertical: None,
        }
    }

    fn view(&self) -> Isometry3<f32> {
        match self.target {
            Some(target) => Isometry3::look_at_rh(&self.position, &target, &self.up),
            None => Isometry3::look_at_rh(&self.position, &(self.position + self.direction), &self.up),
        }
    }

    fn handle_keys(&mut self, input: glutin::KeyboardInput) -> bool {
        //TODO actual fsm
        let mut handled = false;
        match input.scancode {
            16 => {
                if input.state == glutin::ElementState::Pressed {
                    if self.target.is_none() {
                        self.target = Some(Point3::new(0.0, 0.0, 10.0));
                        handled = true;
                    } else {
                        self.target = None;
                    }
                }
            },
            17 => {
                if input.state == glutin::ElementState::Pressed {
                    if self.vertical.is_none() {
                        self.vertical = Some(MoveVertical::Up);
                        handled = true;
                    }
                } else {
                    self.vertical = None;
                    handled = true;
                }
            },
            30 => {
                if input.state == glutin::ElementState::Pressed {
                    if self.horizontal.is_none() {
                        self.horizontal = Some(MoveHorizontal::Left);
                        handled = true;
                    }
                } else {
                    self.horizontal = None;
                    handled = true;
                }
            },
            31 => {
                if input.state == glutin::ElementState::Pressed {
                    if self.vertical.is_none() {
                        self.vertical = Some(MoveVertical::Down);
                        handled = true;
                    }
                } else {
                    self.vertical = None;
                    handled = true;
                }
            },
            32 => {
                if input.state == glutin::ElementState::Pressed {
                    if self.horizontal.is_none() {
                        self.horizontal = Some(MoveHorizontal::Right);
                        handled = true;
                    }
                } else {
                    self.horizontal = None;
                    handled = true;
                }
            },
            _ => (),
        }

        handled
    }

    fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        if self.target.is_none() {
            let rotation = Matrix4::from_scaled_axis(self.up * delta_x * self.projection.fovy())
                * Matrix4::from_scaled_axis(self.direction.cross(&self.up) * delta_y * self.projection.fovy());
            self.direction = rotation.transform_vector(&self.direction);
        }
    }

    // TODO replace with ECS later
    fn step(&mut self, time_step: time::Duration) {
        if let Some(target) = self.target {
            //update direction from target
            self.direction = (target - self.position).normalize();
        }

        //move position from direction
        match &self.vertical {
            Some(dir) => match dir {
                MoveVertical::Up => self.position += self.direction * 0.16,
                MoveVertical::Down => self.position -= self.direction * 0.16,
            },
            None => ()
        }

        match &self.horizontal {
            Some(dir) => match dir {
                MoveHorizontal::Left => self.position -= self.direction.cross(&self.up) * 0.16,
                MoveHorizontal::Right => self.position += self.direction.cross(&self.up) * 0.16,
            },
            None => ()
        }
    }
}


// Vertex data
static VERTEX_DATA: [GLfloat; 72] = [-0.5, -0.5, -0.5,
                                      0.5, -0.5, -0.5,
                                     -0.5, -0.5,  0.5,
                                      0.5, -0.5,  0.5,

                                     -0.5, -0.5,  0.5,
                                      0.5, -0.5,  0.5,
                                     -0.5,  0.5,  0.5,
                                      0.5,  0.5,  0.5,

                                     -0.5, -0.5, -0.5,
                                     -0.5, -0.5,  0.5,
                                     -0.5,  0.5, -0.5,
                                     -0.5,  0.5,  0.5,

                                      0.5, -0.5, -0.5,
                                      0.5, -0.5,  0.5,
                                      0.5,  0.5, -0.5,
                                      0.5,  0.5,  0.5,

                                     -0.5, -0.5, -0.5,
                                      0.5, -0.5, -0.5,
                                     -0.5,  0.5, -0.5,
                                      0.5,  0.5, -0.5,
                                     
                                     -0.5,  0.5, -0.5,
                                      0.5,  0.5, -0.5,
                                     -0.5,  0.5,  0.5,
                                      0.5,  0.5,  0.5];

static INDICES: [GLuint; 36] = [0, 1, 2,
                                1, 2, 3,

                                4, 5, 6,
                                5, 6, 7,

                                8, 9, 10,
                                9, 10, 11,

                                12, 13, 14,
                                13, 14, 15,

                                16, 17, 18,
                                17, 18, 19,

                                20, 21, 22,
                                21, 22, 23];



// Shader sources
static VS_SRC: &'static str = "
#version 330
in vec3 position;
out vec4 color;

uniform mat4 world;

void main() {
    color = vec4(clamp(position, 0.0, 1.0), 1.0);
    gl_Position = world * vec4(position, 1.0);
}";

static FS_SRC: &'static str = "
#version 330
in  vec4 color;
out vec4 out_color;

void main() {
    out_color = color;
}";

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}

fn main() {
    let mut window_size = LogicalSize::new(1024.0, 768.0);
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!")
        .with_dimensions(window_size);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
    }

    unsafe {
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Enable(gl::DEPTH_TEST);
    }

    // Create GLSL shaders
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;
    let mut ibo = 0;
    let world_location;
    let pos_attr;
    let mut scale:f32 = 0.0;

    let mut camera = Camera::new(16.0/9.0,
                                 3.14 / 4.0,
                                 0.1,
                                 10000.0);

    unsafe {
        // Create Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&VERTEX_DATA[0]),
            gl::STATIC_DRAW,
        );
        // Use shader program
        gl::UseProgram(program);
        gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());

        // Specify the layout of the vertex data
        pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr()) as GLuint;
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
            pos_attr,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            0,
            ptr::null(),
        );

        // Create an index buffer and copy indices
        gl::GenBuffers(1, &mut ibo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDICES.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
            mem::transmute(&INDICES[0]),
            gl::STATIC_DRAW,
        );
        
        world_location = gl::GetUniformLocation(program, CString::new("world").unwrap().as_ptr());
        if world_location == -1 {
            println!("Bad uniform location... (handle this?)");
        }
        
    }

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        window_size = logical_size;
                        let dpi_factor = gl_window.get_hidpi_factor();
                        let physical_size = window_size.to_physical(dpi_factor);
                        gl_window.resize(physical_size);
                        unsafe {
                            gl::Viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                        }
                        camera.projection.set_aspect((window_size.width / window_size.height) as f32);
                    },
                    glutin::WindowEvent::HiDpiFactorChanged(dpi_factor) => {
                        let physical_size = window_size.to_physical(dpi_factor);
                        gl_window.resize(physical_size);
                        unsafe {
                            gl::Viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                        }
                    },
                    _ => ()
                },
                glutin::Event::DeviceEvent{ event, .. } => match event {
                    glutin::DeviceEvent::Key(input) => {
                        camera.handle_keys(input);
                    },
                    glutin::DeviceEvent::MouseMotion{delta} => {
                        let delta_x: f32 = (delta.0 / window_size.width) as f32;
                        let delta_y: f32 = (delta.1 / window_size.height) as f32;
                        camera.rotate(delta_x, delta_y);
                    },
                    _ => ()
                },
                _ => ()
            }
        });

        camera.step(time::Duration::from_millis(16));
        //println!("Camera: {:?}", camera);

        scale += 0.01;
        let trans = Vector3::new(scale.sin(), (2.0*scale).sin(), 10.0);
        //let model = Matrix4::from_euler_angles(scale, scale*1.2, 0.0)
        //    .append_translation(&trans);
        let model = Matrix4::new_translation(&trans);

        let view = camera.view().to_homogeneous();

        let model_view_projection = camera.projection.unwrap() * view * model;

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            gl::UniformMatrix4fv(world_location, 1, gl::FALSE, model_view_projection.as_slice().as_ptr());

            gl::EnableVertexAttribArray(pos_attr);

            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, ptr::null());

            gl::DisableVertexAttribArray(pos_attr);
        }

        gl_window.swap_buffers().unwrap();
    }

    // Cleanup
    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
