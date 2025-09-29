// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;

use std::ffi::CString;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

use crate::mesh::{Helicopter, Mesh};

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Helper functions to simplify handling of vao's
unsafe fn create_vao_from_mesh(mesh: &mesh::Mesh) -> (u32, i32){
    let vao = create_vao(&mesh.vertices, &mesh.colors, &mesh.indices, &mesh.normals);
    (vao, mesh.index_count)
}

unsafe fn draw_vao(vao: u32, index_count: i32){
    gl::BindVertexArray(vao);
    gl::DrawElements(gl::TRIANGLES, index_count, gl::UNSIGNED_INT, std::ptr::null())
}

unsafe fn create_vao(vertices: &Vec<f32>, vertices_color: &Vec<f32>, indices: &Vec<u32>, normals: &Vec<f32>) -> u32 {
    let mut vao: u32 = 0;
    gl::GenVertexArrays(1, &mut vao);  
    gl::BindVertexArray(vao);

    let mut vbo_pos: u32 = 0;
    gl::GenBuffers(1, &mut vbo_pos);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_pos);

    gl::BufferData(
        gl::ARRAY_BUFFER, 
        byte_size_of_array(vertices), 
        pointer_to_array(vertices),
        gl::STATIC_DRAW
    );

    let pos_stride = 3 * size_of::<f32>();

    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        pos_stride,
        ptr::null() 
    );
    
    let mut vbo_colors: u32 = 0;
    gl::GenBuffers(1, &mut vbo_colors);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_colors);
    
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertices_color), 
        pointer_to_array(vertices_color),
        gl::STATIC_DRAW
    );
    
    let color_stride = 4 * size_of::<f32>();
    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(
        1, 
        4,
        gl::FLOAT,
        gl::FALSE,
        color_stride,
        ptr::null() 
    );

    let mut vbo_normals: u32 = 0;
    gl::GenBuffers(1, &mut vbo_normals);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_normals);
    
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(normals), 
        pointer_to_array(normals),
        gl::STATIC_DRAW
    );
    
    let normals_stride = 3 * size_of::<f32>();
    gl::EnableVertexAttribArray(2);
    gl::VertexAttribPointer(
        2, 
        3,
        gl::FLOAT,
        gl::FALSE,
        normals_stride,
        ptr::null() 
    );

    let mut ebo: u32 = 0;
    gl::GenBuffers(1, &mut ebo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices),
        pointer_to_array(indices),
        gl::STATIC_DRAW,
    );
    
    vao
}

// I just assume the color red, and i just use z = 0. So just in xy-plane
fn vertices_and_indices_circle(cx: f32, cy: f32, r: f32) -> (Vec<f32>, Vec<u32>){
    let segments: usize = 16;

    // x, y, z, r, g, b
    let mut vertices = Vec::<f32>::with_capacity((segments + 1) * 6);
    let mut indices  = Vec::<u32>::with_capacity(segments * 3);

    // Center
    vertices.extend_from_slice(&[cx, cy, 0.0, 1.0, 0.0, 0.0]);

    // Constr circlepoints with maths
    let two_pi = std::f32::consts::PI * 2.0;
    for i in 0..segments {
        let t = i as f32 * two_pi / segments as f32;
        let x = cx + r * t.cos();
        let y = cy + r * t.sin();
        vertices.extend_from_slice(&[x, y, 0.0, 1.0, 0.0, 0.0]);
    }

    // Indices
    for i in 0..segments {
        let i1 = (i as u32) + 1;
        let i2 = ((i as u32 + 1) % segments as u32) + 1;
        indices.extend_from_slice(&[0, i1, i2]);
    }

    (vertices, indices)
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        let lunarsurface: Mesh = mesh::Terrain::load("./resources/lunarsurface.obj");
        let helicopter: Helicopter = mesh::Helicopter::load("./resources/helicopter.obj");
        let door = helicopter.door;
        let body = helicopter.body;
        let main_rotor = helicopter.main_rotor;
        let tail_rotor = helicopter.tail_rotor;

        let (lunarsurface_vao, lunarsurface_indexcount) = unsafe{create_vao_from_mesh(&lunarsurface)};
        let (body_vao, body_indexcount) = unsafe{create_vao_from_mesh(&body)};
        let (door_vao, door_indexcount) = unsafe{create_vao_from_mesh(&door)};
        let (main_rotor_vao, main_rotor_indexcount) = unsafe{create_vao_from_mesh(&main_rotor)};
        let (tail_rotor_vao, tail_rotor_indexcount) = unsafe{create_vao_from_mesh(&tail_rotor)};


        // == // Shaders
        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link()
        };
        
        let u_transform_loc = unsafe {
            let name = CString::new("uTransform").unwrap();
            gl::GetUniformLocation(simple_shader.program_id, name.as_ptr())
        };


        // Used to demonstrate keyboard handling for exercise 2.
        let mut _arbitrary_number = 0.0; // feel free to remove


        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = -3.0;
        let mut theta_x: f32 = 0.0;
        let mut theta_y: f32 = 0.0;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::A => {x += delta_time;}
                        VirtualKeyCode::D => {x -= delta_time;}
                        VirtualKeyCode::W => {y -= delta_time;}
                        VirtualKeyCode::S => {y += delta_time;}

                        VirtualKeyCode::Space =>  {z += delta_time;}
                        VirtualKeyCode::LShift => {z -= delta_time;}

                        VirtualKeyCode::Up =>     {theta_x -= delta_time;}
                        VirtualKeyCode::Down =>   {theta_x += delta_time;}
                        VirtualKeyCode::Right =>  {theta_y += delta_time;}
                        VirtualKeyCode::Left =>   {theta_y -= delta_time;}

                        // default handler:
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)
            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // == // Issue the necessary gl:: commands to draw your scene here
                simple_shader.activate();
                
                let mut model = glm::Mat4::identity();
                let translation = glm::translation(&glm::vec3(x, y, z));
                model = translation * model;

                let rotate_x_axis = glm::rotate_x(&model, theta_x);
                let rotate_y_axis = glm::rotate_y(&model, theta_y);
                model = rotate_x_axis* rotate_y_axis * model;

                let projection: glm::Mat4 = glm::perspective(window_aspect_ratio, 0.6, 1.0, 1000.0);

                model = projection * model;

                gl::UniformMatrix4fv(u_transform_loc, 1, gl::FALSE, model.as_ptr());

                gl::DepthMask(gl::FALSE); // Disable while drawing, then enable again
                draw_vao(lunarsurface_vao, lunarsurface_indexcount);
                draw_vao(body_vao, body_indexcount);
                draw_vao(door_vao, door_indexcount);
                draw_vao(main_rotor_vao, main_rotor_indexcount);
                draw_vao(tail_rotor_vao, tail_rotor_indexcount);
                gl::DepthMask(gl::TRUE);

            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
