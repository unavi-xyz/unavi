# @wired-xr/scene

A library for creating / loading 3d scenes.

Rendered using `react-three-fiber`.

## Scene

A scene is a 3d environment, stored in a standardized JSON format.

Scenes are made up of primitive objects, such as "Box" and "Sphere".

The idea is that you can safely load any untrusted scene into your app. The scene only describes itself at a high level through primitive objects. It is up to the client to turn the scene JSON into actual 3d objects. This allows the client to limit what the scene can do.

For example, maybe the client limits how many objects can be spawned in to prevent crashing. Or maybe the client only loads assets that are hosted over IPFS, to prevent the potential tracking of people who load the scene.

Currently only static scenes are supported, but there are plans for dynamic physics as well as scripting.
