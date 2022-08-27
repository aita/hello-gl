use std::ffi::CStr;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
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

    unsafe {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);
        gl::BindVertexArray(vao);

        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (std::mem::size_of::<Vertex>() * VERTICES.len()) as isize,
            VERTICES.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<Vertex>() as i32,
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        assert_ne!(vertex_shader, 0);

        gl::ShaderSource(
            vertex_shader,
            1,
            &(VERT_SHADER.as_bytes().as_ptr().cast()),
            &(VERT_SHADER.len().try_into().unwrap()),
        );
        gl::CompileShader(vertex_shader);

        let mut success = 0;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut buf: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(vertex_shader, 1024, &mut log_len, buf.as_mut_ptr().cast());
            buf.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&buf));
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        gl::ShaderSource(
            fragment_shader,
            1,
            &(FRAG_SHADER.as_bytes().as_ptr().cast()),
            &(FRAG_SHADER.len().try_into().unwrap()),
        );
        gl::CompileShader(fragment_shader);

        let mut success = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut buf: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(fragment_shader, 1024, &mut log_len, buf.as_mut_ptr().cast());
            buf.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&buf));
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut buf: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetProgramInfoLog(shader_program, 1024, &mut log_len, buf.as_mut_ptr().cast());
            buf.set_len(log_len.try_into().unwrap());
            panic!("Program Link Error: {}", String::from_utf8_lossy(&buf));
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        gl::UseProgram(shader_program);
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
