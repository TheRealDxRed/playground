use gl::types::{GLfloat, GLuint, GLenum, GLint, GLchar, GLsizeiptr};
use libc::c_void;
use obj::{Position, load_obj, Obj};
use sdl2::{event::Event, keyboard::Keycode};
use std::{mem, ffi::CString, ptr, str, io::{BufReader, Cursor}};

fn main() -> Result<(),String> {
	let teapot_obj = include_bytes!("teapot.obj").to_vec();
	let teapot_input = BufReader::new(Cursor::new(teapot_obj));
	let teapot: Obj<Position, GLuint> = load_obj(teapot_input).unwrap();

	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;

	video_subsystem.gl_attr().set_context_major_version(3); // Version major (X).Y
	video_subsystem.gl_attr().set_context_minor_version(1);	// Version minor X.(Y)
	video_subsystem.gl_attr().set_double_buffer(true);			// VSync
	video_subsystem.gl_attr().set_depth_size(24);						// Bits per pixel

	let window = video_subsystem
		.window(
			"Rust SDL2 Window",
			800,
			600
		)
		.position_centered()
		.opengl()
		.build()
		.map_err(|e| e.to_string())?;

	let mut event_pump = sdl_context.event_pump()?;

	// opengl init goes here
	let mut context = window.gl_create_context().unwrap();
	window.gl_make_current(&mut context)?;

	gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const c_void);

	video_subsystem.gl_set_swap_interval(1)?;

	let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
	let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);

	let program = link_program(vs, fs);

	let mut vao = 0;
	let mut vbo = 0;

	unsafe {
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);

		gl::GenBuffers(1, &mut vbo);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(teapot.vertices.len() * 3 * mem::size_of::<GLfloat>()) as GLsizeiptr,
			mem::transmute(&teapot.vertices[0]),
			gl::STATIC_DRAW
		);

		gl::UseProgram(program);

		#[allow(temporary_cstring_as_ptr)]
		gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());

		#[allow(temporary_cstring_as_ptr)]
		let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());

		gl::EnableVertexAttribArray(pos_attr as GLuint);
		gl::VertexAttribPointer(
			pos_attr as GLuint,
			2,
			gl::FLOAT,
			gl::FALSE,
			0,
			ptr::null()
		);
	}

	'running: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. }
				| Event::KeyDown {
					keycode: Some(Keycode::Escape),
					..
				} => break 'running,
				_ => {}
			}
		}

		unsafe {
			gl::ClearColor(0.3, 0.3, 0.3, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::DrawArrays(gl::TRIANGLES, 0, 3);
		}

		window.gl_swap_window();
	}

	Ok(())
}

static VS_SRC: &'static str = include_str!("test.vs");
static FS_SRC: &'static str = include_str!("test.fs");

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
	let shader;
	unsafe {
		shader = gl::CreateShader(ty);

		let c_str = CString::new(src.as_bytes()).unwrap();
		gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
		gl::CompileShader(shader);

		let mut status = gl::FALSE as GLint;
		gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

		if status != (gl::TRUE as GLint) {
			let mut len = 0;
			gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = Vec::new();
			buf.set_len((len as usize) - 1);
			gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
			panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ShaderInfoLog not valid utf8"));
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

		let mut status = gl::FALSE as GLint;
		gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

		if status != (gl::TRUE as GLint) {
			let mut len: GLint = 0;
			gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = Vec::new();
			buf.set_len((len as usize) - 1);
			gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
			panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ProgramInfoLog not valid utf8"));
		}
		program
	}
}