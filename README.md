# iced_blur

> [!WARNING]
> This is currently a prototype, it may not be compatible with your hardware or crash your application.

a blur widget for [iced](https://github.com/iced-rs/iced/)

## Implementation

Implemented based on the [Bandwith-Efficient Rendering](https://community.arm.com/cfs-file/__key/communityserver-blogs-components-weblogfiles/00-00-00-20-66/siggraph2015_2D00_mmg_2D00_marius_2D00_notes.pdf) notes from SIGGRAPH 2015.

The implementation is very simple, it copies the frambuffer texture into a new texture, it then performs the ping-pong downsampling and upsampling described in the notes, and the blits the resulting blurred texture into the framebuffer.

## Limitations

Currently this requires the following diff on `iced`
```
--- a/wgpu/src/window/compositor.rs
+++ b/wgpu/src/window/compositor.rs
@@ -320,7 +320,8 @@ impl graphics::Compositor for Compositor {
         surface.configure(
             &self.engine.device,
             &wgpu::SurfaceConfiguration {
-                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
+                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
+                    | wgpu::TextureUsages::COPY_SRC,
                 format: self.format,
                 present_mode: self.settings.present_mode,
                 width,
```

this allows the framebuffer to be used as a copy target, this might be impact performance and not be supported on all platforms/hardware.
