use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

pub trait WgpuApplication: Sized {
    fn window_attributes() -> anyhow::Result<Option<winit::window::WindowAttributes>>;
    fn new(
        window: Arc<winit::window::Window>,
    ) -> impl std::future::Future<Output = anyhow::Result<Self>> + Send;
    fn window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
}

/// WgpuAppContainer is a container that handles the abstraction over standard window based applications,
/// and wasm32 applications. It also handles the resume cycle for platforms (mobile) where that is relevant.
/// Window events are forwarded to the App struct.
struct WgpuAppContainer<AppType: WgpuApplication> {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<App>>,
    state: Option<AppType>,
    window_attributes: winit::window::WindowAttributes,
}

impl<T: WgpuApplication> WgpuAppContainer<T> {
    pub fn new(
        window_attributes: winit::window::WindowAttributes,
        #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<App>,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy,
            state: None,
            window_attributes,
        }
    }
}

impl<AppType: WgpuApplication + 'static> ApplicationHandler<AppType> for WgpuAppContainer<AppType> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = self.window_attributes.clone();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the
            self.state = Some(pollster::block_on(AppType::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                State::new(window)
                                    .await
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: AppType) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            state.window_event(event_loop, event);
        }
    }
}

pub fn run<AppType: WgpuApplication + 'static>() -> anyhow::Result<()> {
    // Get the window attributes from the app. It can return Ok(None) if the app doesn't need a window.
    // Or it can return an error if something went wrong.
    let window_attributes = match AppType::window_attributes()? {
        Some(attributes) => attributes,
        None => return Ok(()),
    };

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = WgpuAppContainer::<AppType>::new(window_attributes);
        event_loop.run_app(&mut app)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let app = WgpuAppContainer::new(window_attributes, &event_loop);
        event_loop.spawn_app(app);
    }

    Ok(())
}
