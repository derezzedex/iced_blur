# iced_blur

> [!WARNING]
> This is currently a prototype, it may incompatible with your hardware and/or might crash your application.

a blur widget for [iced](https://github.com/iced-rs/iced/)

## Implementation

Implemented based on the [Bandwith-Efficient Rendering](https://community.arm.com/cfs-file/__key/communityserver-blogs-components-weblogfiles/00-00-00-20-66/siggraph2015_2D00_mmg_2D00_marius_2D00_notes.pdf) notes from SIGGRAPH 2015.

The implementation is very simple, it copies the ([offscreen](#limitations)) framebuffer texture into a new texture, it then performs the ping-pong downsampling and upsampling described in the notes, and then blits the resulting blurred texture into the framebuffer.

## Limitations

Currently this requires [a fork](https://github.com/derezzedex/iced/tree/dev/offscreen) of `iced`, with the following [diff](https://github.com/iced-rs/iced/compare/master...derezzedex:iced:dev/offscreen), which makes `iced` always draw to an offscreen buffer. This removes the need of requiring the framebuffer being "copyable".

Although this doesn't have been properly tested (hence a fork and not a PR yet), this is widely done on games and UIs to enable composable effects, so it should be reasonably safe and portable.
