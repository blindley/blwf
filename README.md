## BLWF (bl wgpu/winit framework)

A framework for building applications with wgpu and winit.

# Mimimal Example

```rust
use blwf::{wgpu, winit};

use blwf::{WgpuApplication, WgpuBase, run};
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

fn main() -> blwf::Result<()> {
    let mut frames_processed: u32 = 0;
    run::<MinimalWgpuApp>(
        winit::window::WindowAttributes::default()
            .with_title("Minimal WGPU App")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
        &mut frames_processed,
    )?;

    println!("frames_processed: {frames_processed}");

    Ok(())
}

struct MinimalWgpuApp {
    base: WgpuBase,
}

impl WgpuApplication for MinimalWgpuApp {
    type Data = u32;

    async fn new(window: Arc<Window>, data: &mut Self::Data) -> blwf::Result<Self> {
        let base = WgpuBase::new(window).await?;
        Ok(Self { base })
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: WindowEvent,
        data: &mut Self::Data,
    ) {
        use winit::event::KeyEvent;
        use winit::keyboard::KeyCode;
        use winit::keyboard::PhysicalKey;

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.base.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                *data += 1;
                if let Err(e) = self.render() {
                    eprintln!("{e}");
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}

impl MinimalWgpuApp {
    fn render(&mut self) -> blwf::Result<()> {
        if let Some(surface_texture) = self.base.begin_render()? {
            let view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder =
                self.base
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Command Encoder"),
                    });

            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
            }

            self.base.queue.submit(std::iter::once(encoder.finish()));
            self.base.end_render(surface_texture)?;
        }

        Ok(())
    }
}

```
