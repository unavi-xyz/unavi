# @wired-xr/examples

Examples for @wired-xr/engine.

Note: When run in development mode, memory usage will be increased significantly. If you are looking for accurate stats, run the example in production mode (`yarn build && yarn start` as opposed to `yarn dev`).

Models taken from [glTF-Sample-Models](https://github.com/KhronosGroup/glTF-Sample-Models).

## Showcase

Models demonstrating features of the core glTF specification. Some models may include versions with extensions, e.g. copies with or without Draco compression.

| Model                                           |                              Screenshot                               |     Normal Map     |   Occlusion Map    |    Emissive Map    |
| ----------------------------------------------- | :-------------------------------------------------------------------: | :----------------: | :----------------: | :----------------: |
| [Antique Camera](public/models/AntiqueCamera)   | ![screenshot](public/models/AntiqueCamera/screenshot/screenshot.png)  | :white_check_mark: |                    |                    |
| [Avocado](public/models/Avocado)                |    ![screenshot](public/models/Avocado/screenshot/screenshot.jpg)     | :white_check_mark: |                    |                    |
| [Barramundi Fish](public/models/BarramundiFish) | ![screenshot](public/models/BarramundiFish/screenshot/screenshot.jpg) | :white_check_mark: | :white_check_mark: |                    |
| [Boom Box](public/models/BoomBox)               |    ![screenshot](public/models/BoomBox/screenshot/screenshot.jpg)     | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| [Corset](public/models/Corset)                  |     ![screenshot](public/models/Corset/screenshot/screenshot.jpg)     | :white_check_mark: | :white_check_mark: |                    |
| [Damaged Helmet](public/models/DamagedHelmet)   | ![screenshot](public/models/DamagedHelmet/screenshot/screenshot.png)  | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| [Flight Helmet](public/models/FlightHelmet)     |  ![screenshot](public/models/FlightHelmet/screenshot/screenshot.jpg)  | :white_check_mark: | :white_check_mark: |                    |
| [Lantern](public/models/Lantern)                |    ![screenshot](public/models/Lantern/screenshot/screenshot.jpg)     | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| [Sci Fi Helmet](public/models/SciFiHelmet)      |  ![screenshot](public/models/SciFiHelmet/screenshot/screenshot.jpg)   | :white_check_mark: | :white_check_mark: |                    |
| [Suzanne](public/models/Suzanne)                |    ![screenshot](public/models/Suzanne/screenshot/screenshot.jpg)     |                    |                    |                    |
| [Water Bottle](public/models/WaterBottle)       |  ![screenshot](public/models/WaterBottle/screenshot/screenshot.jpg)   | :white_check_mark: | :white_check_mark: | :white_check_mark: |

### Feature Tests

Models meant to easily illustrate and test specific features of the core specification.

| Model                                                                             | Screenshot                                                                            | Description                                                                                      |
| --------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| [Alpha Blend Mode Test](public/models/AlphaBlendModeTest)                         | ![screenshot](public/models/AlphaBlendModeTest/screenshot/screenshot.png)             | Tests alpha modes and settings.                                                                  |
| [Boom Box With Axes](public/models/BoomBoxWithAxes)                               | ![screenshot](public/models/BoomBoxWithAxes/screenshot/screenshot.jpg)                | Shows X, Y, and Z axis default orientations.                                                     |
| [Metal Rough Spheres](public/models/MetalRoughSpheres)                            | ![screenshot](public/models/MetalRoughSpheres/screenshot/screenshot.png)              | Tests various metal and roughness values (texture mapped).                                       |
| [Metal Rough Spheres (Textureless)](public/models/MetalRoughSpheresNoTextures)    | ![screenshot](public/models/MetalRoughSpheresNoTextures/screenshot/screenshot.png)    | Tests various metal and roughness values (textureless).                                          |
| [Morph Primitives Test](public/models/MorphPrimitivesTest)                        | ![screenshot](public/models/MorphPrimitivesTest/screenshot/screenshot.jpg)            | Tests a morph target on multiple primitives.                                                     |
| [Morph Stress Test](public/models/MorphStressTest)                                | ![screenshot](public/models/MorphStressTest/screenshot/screenshot.jpg)                | Tests up to 8 morph targets.                                                                     |
| [Multi UV Test](public/models/MultiUVTest)                                        | ![screenshot](public/models/MultiUVTest/screenshot/screenshot.jpg)                    | Tests a second set of texture coordinates.                                                       |
| [Normal Tangent Test](public/models/NormalTangentTest)                            | ![screenshot](public/models/NormalTangentTest/screenshot/screenshot.png)              | Tests an engine's ability to automatically generate tangent vectors for a normal map.            |
| [Normal Tangent Mirror Test](public/models/NormalTangentMirrorTest)               | ![screenshot](public/models/NormalTangentMirrorTest/screenshot/screenshot.png)        | Tests an engine's ability to load supplied tangent vectors for a normal map.                     |
| [Orientation Test](public/models/OrientationTest)                                 | ![screenshot](public/models/OrientationTest/screenshot/screenshot.png)                | Tests node translations and rotations.                                                           |
| [Recursive Skeletons](public/models/RecursiveSkeletons)                           | ![screenshot](public/models/RecursiveSkeletons/screenshot/screenshot.jpg)             | Tests unusual skinning cases with reused meshes and recursive skeletons.                         |
| [Texture Coordinate Test](public/models/TextureCoordinateTest)                    | ![screenshot](public/models/TextureCoordinateTest/screenshot/screenshot.png)          | Shows how XYZ and UV positions relate to displayed geometry.                                     |
| [Texture Linear Interpolation Test](public/models/TextureLinearInterpolationTest) | ![screenshot](public/models/TextureLinearInterpolationTest/screenshot/screenshot.png) | Tests that linear texture interpolation is performed on linear values, i.e. after sRGB decoding. |
| [Texture Settings Test](public/models/TextureSettingsTest)                        | ![screenshot](public/models/TextureSettingsTest/screenshot/screenshot.png)            | Tests single/double-sided and various texturing modes.                                           |
| [Vertex Color Test](public/models/VertexColorTest)                                | ![screenshot](public/models/VertexColorTest/screenshot/screenshot.png)                | Tests if vertex colors are supported.                                                            |

## Minimal Tests

Minimal models testing very narrow pieces of the core specification.

| Model                                                            | Screenshot                                                                    | Description                                                                                                                                                                                                                                         |
| ---------------------------------------------------------------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Triangle Without Indices](public/models/TriangleWithoutIndices) | ![screenshot](public/models/TriangleWithoutIndices/screenshot/screenshot.png) | The simplest possible glTF asset: A single `scene` with a single `node` and a single `mesh` with a single `mesh.primitive` with a single triangle with a single attribute, without indices and without a `material`                                 |
| [Triangle](public/models/Triangle)                               | ![screenshot](public/models/Triangle/screenshot/screenshot.png)               | A very simple glTF asset: The basic structure is the same as in [Triangle Without Indices](TriangleWithoutIndices), but here, the `mesh.primitive` describes an _indexed_ geometry                                                                  |
| [Animated Triangle](public/models/AnimatedTriangle)              | ![screenshot](public/models/AnimatedTriangle/screenshot/screenshot.gif)       | This sample is similar to the [Triangle](Triangle), but the `node` has a `rotation` property that is modified with a simple `animation`                                                                                                             |
| [Animated Morph Cube](public/models/AnimatedMorphCube)           | ![screenshot](public/models/AnimatedMorphCube/screenshot/screenshot.gif)      | Demonstrates a simple cube with two simple morph targets and an animation that transitions between them both.                                                                                                                                       |
| [Animated Morph Sphere](public/models/AnimatedMorphSphere)       | ![screenshot](public/models/AnimatedMorphSphere/screenshot/screenshot.gif)    | This sample is similar to the [Animated Morph Cube](AnimatedMorphCube), but the two morph targets move many more vertices and are more extreme than with the cube.                                                                                  |
| [Simple Meshes](public/models/SimpleMeshes)                      | ![screenshot](public/models/SimpleMeshes/screenshot/screenshot.png)           | A simple `scene` with two `nodes`, both containing the same `mesh`, namely a `mesh` with a single `mesh.primitive` with a single indexed triangle with _multiple_ attributes (positions, normals and texture coordinates), but without a `material` |
| [Simple Morph](public/models/SimpleMorph)                        | ![screenshot](public/models/SimpleMorph/screenshot/screenshot.png)            | A triangle with a morph animation applied                                                                                                                                                                                                           |
| [Simple Sparse Accessor](public/models/SimpleSparseAccessor)     | ![screenshot](public/models/SimpleSparseAccessor/screenshot/screenshot.png)   | A simple mesh that uses sparse accessors                                                                                                                                                                                                            |
| [Simple Skin](public/models/SimpleSkin)                          | ![screenshot](public/models/SimpleSkin/screenshot/screenshot.gif)             | A simple example of vertex skinning in glTF                                                                                                                                                                                                         |
| [Interpolation Test](public/models/InterpolationTest)            | ![screenshot](public/models/InterpolationTest/screenshot/screenshot.gif)      | A sample with three different `animation` interpolations                                                                                                                                                                                            |
| [Unicode Test](Unicode❤♻Test)                                    | ![screenshot](public/models/Unicode❤♻Test/screenshot/screenshot.png)          | A sample with Unicode characters in file, material, and mesh names                                                                                                                                                                                  |
