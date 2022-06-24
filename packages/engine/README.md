# @wired-xr/engine

A 3d game engine for The Wired.

Uses [Threejs](https://github.com/mrdoob/three.js) / [React Three Fiber](https://github.com/pmndrs/react-three-fiber) for rendering.

## About

### Scenes

A scene is a 3d environment. Scenes are made up of primitive objects, such as `Box` and `Sphere`.

**The idea is that you can safely load any untrusted scene into your app**. The scene only describes itself at a high level through primitive objects. It is up to the client to implement these primitives and turn the scene JSON into run code. This allows the client to limit what the scene can do.

For example, maybe the client limits how many objects can be spawned in to prevent crashing. Or maybe the client only loads assets that are hosted over IPFS, to prevent the potential tracking of people who load the scene.

Currently only static scenes are supported, but there are plans for dynamic physics, scripting, and much more.
