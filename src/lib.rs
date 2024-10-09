use glam::IVec2;
use logic::Rusteroids;
use renderer::Renderer;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
mod camera;
mod logic;
mod mesh;
mod renderer;
mod utils;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(512, 512));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    // Game logic
    let mut game_logic = Rusteroids::new();
    let (mut is_w_pressed, mut is_a_pressed, mut is_d_pressed) = (false, false, false);

    // Create the Renderer
    let mut renderer = Renderer::new(&window).await;
    // Add the player meshh
    renderer.add_mesh(utils::WEDGE);

    let mut surface_configured = false;

    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == renderer.window().id() => {
                    if !renderer.input(event) {
                        // UPDATED!
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Pressed,
                                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => control_flow.exit(),
                            WindowEvent::KeyboardInput { event, .. } => {
                                match event.physical_key {
                                    PhysicalKey::Code(KeyCode::KeyW) => {
                                        is_w_pressed = matches!(event.state, ElementState::Pressed)
                                    }
                                    PhysicalKey::Code(KeyCode::KeyA) => {
                                        is_a_pressed = matches!(event.state, ElementState::Pressed)
                                    }
                                    PhysicalKey::Code(KeyCode::KeyD) => {
                                        is_d_pressed = matches!(event.state, ElementState::Pressed)
                                    }
                                    _ => {}
                                };
                                game_logic.update_keys(is_w_pressed, is_a_pressed, is_d_pressed);
                            }
                            WindowEvent::Resized(physical_size) => {
                                log::info!("physical_size: {physical_size:?}");
                                surface_configured = true;
                                game_logic.set_bounds(IVec2::new(
                                    physical_size.width as i32,
                                    physical_size.height as i32,
                                ));
                                renderer.resize(*physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                renderer.window().request_redraw();

                                if !surface_configured {
                                    return;
                                }
                                game_logic.tick();
                                renderer.update(&game_logic.get_battleship_model_matrix());
                                match renderer.render() {
                                    Ok(_) => {}
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => {
                                        let size = renderer.get_size();
                                        renderer.resize(size)
                                    }
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
