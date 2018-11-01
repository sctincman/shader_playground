# Shader Playground

A Rust + OpenGL based graphics playground. Exercise in raw graphics/OpenGL programming and attempt to learn Rust.

This is currently targeting gl-rs in order to deepen my knowledge of the OpenGL C API, and loosely using [OGL Modern OpenGL programming](http://ogldev.atspace.co.uk/index.html) to guide feature development, but adapting for Rust and personal goals.

## Plans

* ~~Camera control via buttons and mouse~~
* Load vertex and fragment shaders from files.
* Watch shader files for changes and recompile on the fly.
* Scene support (either multiple OBJ files or GLTF loader).

## Usage

Run with `cargo run`.

### Controls

* WASD to move
    * W/S move towards/away from target/direction
    * A/D move left/right in the XZ plane
* Q to lock/unlock view from target (currently only the cube).
* Mouse to move view direction when unlocked

## Building

Build with `cargo build`
