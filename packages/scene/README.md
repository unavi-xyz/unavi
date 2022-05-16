# @wired-xr/scene

A library for creating / loading 3d scenes.

Rendered using `react-three-fiber`.

## Scene

A scene is a 3d environment, stored in a standardized JSON format.

Scenes roughly follow an Entity-Component System structure, and will have support for scripting.

## WIP

The idea is that you can load any untrusted scene into your app without risk of crashing or privacy risks. Scene code will run in a web worker, and limit how many objects are loaded into the canvas to prevent spam / crashing.

Still figuring it out.
