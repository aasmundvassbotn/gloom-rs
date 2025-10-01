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
mod scene_graph;
use scene_graph::SceneNode;
mod toolbox;


use std::ffi::CString;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

use crate::mesh::{Helicopter, Mesh};
use crate::toolbox::Heading;

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

fn apply_heading(helicopter_body: &mut SceneNode, time: f32) {
    let heading: Heading = toolbox::simple_heading_animation(time);
    helicopter_body.position.x = heading.x;
    helicopter_body.position.z = heading.z;

    helicopter_body.rotation.z = heading.roll;
    helicopter_body.rotation.x = heading.pitch;
    helicopter_body.rotation.y = heading.yaw;
}

unsafe fn draw_scene(
    node: &scene_graph::SceneNode,
    view_projection_matrix: &glm::Mat4,
    transformation_so_far: &glm::Mat4,
    u_model_loc: i32,
    u_view_loc: i32,
    ) {
    let t = glm::translation(&node.position);

    let mut r: glm::Mat4 = glm::identity();
    r = glm::rotate_x(&r, node.rotation.x);
    r = glm::rotate_y(&r, node.rotation.y);
    r = glm::rotate_z(&r, node.rotation.z);

    let to_pivot     = glm::translation(&node.reference_point);
    let from_pivot   = glm::translation(&(-node.reference_point));
    let local = t * to_pivot * r * from_pivot;
    let model = transformation_so_far * local;

    gl::UniformMatrix4fv(u_model_loc, 1, gl::FALSE, model.as_ptr());
    gl::UniformMatrix4fv(u_view_loc, 1, gl::FALSE, view_projection_matrix.as_ptr());

    if node.index_count > 0 {
        draw_vao(node.vao_id, node.index_count);
    }
    for &child in &node.children {
        draw_scene(
            &*child, 
            view_projection_matrix, 
            &model, 
            u_model_loc, 
            u_view_loc
        );
    }
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
        let door: Mesh = helicopter.door;
        let body: Mesh = helicopter.body;
        let main_rotor: Mesh = helicopter.main_rotor;
        let tail_rotor: Mesh = helicopter.tail_rotor;

        let (lunarsurface_vao, lunarsurface_indexcount) = unsafe{create_vao_from_mesh(&lunarsurface)};
        let (body_vao, body_indexcount) = unsafe{create_vao_from_mesh(&body)};
        let (door_vao, door_indexcount) = unsafe{create_vao_from_mesh(&door)};
        let (main_rotor_vao, main_rotor_indexcount) = unsafe{create_vao_from_mesh(&main_rotor)};
        let (tail_rotor_vao, tail_rotor_indexcount) = unsafe{create_vao_from_mesh(&tail_rotor)};

        let mut scene = SceneNode::new();
        let lunarsurface_scene = SceneNode::from_vao(lunarsurface_vao, lunarsurface_indexcount);
        scene.add_child(&lunarsurface_scene);

        // We iterate 5 times and create helicopters. The helicopters are stored as children
        let mut i = 0;
        while i < 5 {
            let mut body_scene = SceneNode::from_vao(body_vao, body_indexcount);
            let door_scene = SceneNode::from_vao(door_vao, door_indexcount);
            let main_rotor_scene = SceneNode::from_vao(main_rotor_vao, main_rotor_indexcount);
            let mut tail_rotor_scene = SceneNode::from_vao(tail_rotor_vao, tail_rotor_indexcount);
    
            // Set the tail reference point. This prevents the tail rotor from spinning around the body chassis of the helicopter
            tail_rotor_scene.reference_point = nalgebra_glm::Vec3::new(0.35, 2.3, 10.4);
    
            body_scene.add_child(&door_scene);
            body_scene.add_child(&main_rotor_scene);
            body_scene.add_child(&tail_rotor_scene);
            
            scene.add_child(&body_scene);

            i = i + 1;
        } // After the loop, we essentially have 5 helicopters in the exact same spot

        scene.print();

        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link()
        };
        
        let u_view_loc = unsafe {
            let name = CString::new("modelViewProj").unwrap();
            gl::GetUniformLocation(simple_shader.program_id, name.as_ptr())
        };

        let u_model_loc = unsafe {
            let name = CString::new("model").unwrap();
            gl::GetUniformLocation(simple_shader.program_id, name.as_ptr())
        };

        // Variables for the camera projection
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        let mut x = 50.0;
        let mut y = -3.0;
        let mut z = 50.0;
        let mut theta_x: f32 = 0.2;
        let mut theta_y: f32 = 2.3;

        let mut helicopter_doors_slider_value = 0.0;
        
        // The main rendering loop
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            let helicopter_count = scene.children.len();
            for j in 1..helicopter_count {
                let helicopter = scene.get_child(j);

                // Controll doors
                helicopter.get_child(0).position.z = helicopter_doors_slider_value;

                // Spin rotors
                helicopter.get_child(1).rotation.y = elapsed * 10.0;
                helicopter.get_child(2).rotation.x = elapsed * 10.0;



                apply_heading(helicopter, elapsed + (j as f32 - 1.0) * 0.85);
            }

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

                        VirtualKeyCode::A => {x += delta_time * 7.5;}
                        VirtualKeyCode::D => {x -= delta_time * 7.5;}
                        VirtualKeyCode::W => {y -= delta_time * 7.5;}
                        VirtualKeyCode::S => {y += delta_time * 7.5;}

                        VirtualKeyCode::Space =>  {z += delta_time * 7.5;}
                        VirtualKeyCode::LShift => {z -= delta_time * 7.5;}

                        VirtualKeyCode::Up =>     {theta_x -= delta_time;}
                        VirtualKeyCode::Down =>   {theta_x += delta_time;}
                        VirtualKeyCode::Right =>  {theta_y += delta_time;}
                        VirtualKeyCode::Left =>   {theta_y -= delta_time;}

                        VirtualKeyCode::J => {
                            if helicopter_doors_slider_value < 2.0 {
                                helicopter_doors_slider_value += delta_time * 5.0;
                            }
                        }
                        VirtualKeyCode::K => {
                            if helicopter_doors_slider_value > 0.0 {
                                helicopter_doors_slider_value -= delta_time * 5.0;
                            }
                        }

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

                simple_shader.activate();
                
                let mut view_projection_matrix = glm::Mat4::identity();
                let translation = glm::translation(&glm::vec3(x, y, z));
                let rotate_x_axis = glm::rotate_x(&view_projection_matrix, theta_x);
                let rotate_y_axis = glm::rotate_y(&view_projection_matrix, theta_y);
                let projection: glm::Mat4 = glm::perspective(window_aspect_ratio, 0.6, 1.0, 1000.0);

                view_projection_matrix = projection * rotate_x_axis* rotate_y_axis * translation * view_projection_matrix;

                let translation_so_far = glm::Mat4::identity();
                //gl::DepthMask(gl::FALSE); // Disable while drawing, then enable again
                draw_scene(&scene, &view_projection_matrix, &translation_so_far, u_model_loc, u_view_loc);
                //gl::DepthMask(gl::TRUE);
            }
            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });

    // == // From here on down there are only internals.
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
