use std::ffi::CStr;

use anyhow::{anyhow, Result};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

struct VertexArray(gl::types::GLuint);

impl VertexArray {
    fn new() -> Result<VertexArray> {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        if id == 0 {
            return Err(anyhow!("Failed to create vertex array"));
        } else {
            Ok(VertexArray(id))
        }
    }

    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.0);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

struct Buffer(gl::types::GLuint);

impl Buffer {
    fn new() -> Result<Buffer> {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        if id == 0 {
            Err(anyhow!("Failed to create buffer"))
        } else {
            Ok(Buffer(id))
        }
    }

    fn bind(&self, target: gl::types::GLenum) {
        unsafe {
            gl::BindBuffer(target, self.0);
        }
    }

    fn unbind(&self, target: gl::types::GLenum) {
        unsafe {
            gl::BindBuffer(target, 0);
        }
    }

    fn data(&self, target: gl::types::GLenum, data: &[u8], usage: gl::types::GLenum) {
        unsafe {
            gl::BufferData(
                target,
                data.len() as gl::types::GLsizeiptr,
                data.as_ptr() as *const gl::types::GLvoid,
                usage,
            );
        }
    }
}

struct Shader(gl::types::GLuint);

impl Shader {
    fn from_source(kind: gl::types::GLenum, source: &str) -> Result<Shader> {
        let id = unsafe { gl::CreateShader(kind) };
        if id == 0 {
            return Err(anyhow!("Failed to create shader"));
        } else {
            unsafe {
                gl::ShaderSource(
                    id,
                    1,
                    &(source.as_bytes().as_ptr().cast()),
                    &(source.len().try_into().unwrap()),
                );
                gl::CompileShader(id);

                let mut success = 0;
                gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
                if success == 0 {
                    let mut buf: Vec<u8> = Vec::with_capacity(1024);
                    let mut log_len = 0_i32;
                    gl::GetShaderInfoLog(id, 1024, &mut log_len, buf.as_mut_ptr().cast());
                    buf.set_len(log_len.try_into().unwrap());
                    Err(anyhow!("{:?}", String::from_utf8(buf)))
                } else {
                    Ok(Shader(id))
                }
            }
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

struct Program(gl::types::GLuint);

impl Program {
    fn new() -> Result<Program> {
        let id = unsafe { gl::CreateProgram() };
        if id == 0 {
            return Err(anyhow!("Failed to create program"));
        } else {
            Ok(Program(id))
        }
    }

    fn attach(&self, shader: &Shader) {
        unsafe {
            gl::AttachShader(self.0, shader.0);
        }
    }

    fn link(&self) -> Result<()> {
        unsafe {
            gl::LinkProgram(self.0);

            let mut success = 0;
            gl::GetProgramiv(self.0, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut buf: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                gl::GetProgramInfoLog(self.0, 1024, &mut log_len, buf.as_mut_ptr().cast());
                buf.set_len(log_len.try_into().unwrap());
                Err(anyhow!("{:?}", String::from_utf8(buf)))
            } else {
                Ok(())
            }
        }
    }

    fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }
}

/// Simple loading example
fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("A fantastic window!");

    let windowed_context = ContextBuilder::new()
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    // gl::load_with(|s| window.get_proc_address(s) as *const _);
    gl::load_with(|ptr| windowed_context.get_proc_address(ptr) as *const _);

    let version = unsafe {
        let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };
    println!("OpenGL version {}", version);

    type Vertex = [f32; 3];
    const VERTICES: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];
    const VERT_SHADER: &str = r#"#version 330 core
    layout (location = 0) in vec3 pos;
    void main() {
      gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    }
    "#;
    const FRAG_SHADER: &str = r#"#version 330 core
    out vec4 final_color;

    void main() {
        final_color = vec4(1.0, 0.5, 0.2, 1.0);
    }
    "#;

    let va = VertexArray::new().unwrap();
    va.bind();

    let vb = Buffer::new().unwrap();
    vb.bind(gl::ARRAY_BUFFER);
    vb.data(
        gl::ARRAY_BUFFER,
        bytemuck::cast_slice(&VERTICES),
        gl::STATIC_DRAW,
    );

    unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<Vertex>() as i32,
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);

        let vertex_shader = Shader::from_source(gl::VERTEX_SHADER, VERT_SHADER).unwrap();
        let fragment_shader = Shader::from_source(gl::FRAGMENT_SHADER, FRAG_SHADER).unwrap();

        let program = Program::new().unwrap();
        program.attach(&vertex_shader);
        program.attach(&fragment_shader);
        program.link().unwrap();
        program.use_program();

        vertex_shader.delete();
        fragment_shader.delete();
    }

    event_loop.run(move |event, _, control_flow| {
        // println!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::DrawArrays(gl::TRIANGLES, 0, 3);
                }
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
