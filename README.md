A stateless wrapper around OpenGL, to make it easier to use and more type-safe.

This library is somewhat similar to [glium](https://github.com/glium/glium); the main differences are that this library supports WebGL through `web-sys` while AFAIK glium only supports WebGL through stdweb, this library only implements a subset of OpenGL functionality (though more functionality can be added as needed), and some parts of the API (such as meshes) are somewhat higher-level.

This also includes a simple GUI system, event handling, OpenGL context creation, and a few other utilities.

This is a replacement for my previous libraries `webgl-wrapper` and `webgl-gui`. The API is similar, but some parts (such as events) had to be modified to support GLFW on desktop (separate events are used for a keypress versus character input, because GLFW separates the two). Eventually this should work equally well on WebGL and desktop, but right now the API is slightly different between platforms. It uses the `glow` library so it works on both desktop and wasm.

Current features:

* Programs, meshes, 2D textures, and basic support for framebuffers and renderbuffers
* State caching to reduce the number of redundant OpenGL calls
* Instancing

Features not yet implemented:

* An easier way to implement the `Vertex` and `Uniforms` traits
* More usage examples
* More types of textures
