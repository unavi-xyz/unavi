---
sidebar_position: 40
sidebar_label: 🌏 Spaces
title: Spaces
---

A space is a user-created, 3D multiplayer environment.

Spaces are like 3D websites - instead of clicking on a link and viewing a web page, you walk through a portal and visit a virtual space, and can interact with other people who are also in the space.

## Creation

Spaces are created using the studio - a visual editor similar to Unity. Once your space is ready, you can publish it from the studio and mint it as a [Lens Publication NFTs](https://docs.lens.xyz/docs/publication). This will export the scene as a [glTF](https://www.khronos.org/gltf/) model, and upload it to IPFS.

The exciting thing about glTF is that its an open file format, supported by many engines. Scenes you create in the studio can be opened in 3D editors like Blender, Unity, and Unreal Engine, or other platforms like [Webaverse](https://twitter.com/webaverse).

It also works the other direction - you can import glTF models into the studio and use them to construct your space.

Visit [thewired.space/create](https://www.thewired.space/studio) to get started and create a project.

## Hosting

Just like websites, a space needs a server to host it. The team runs a host server that is automatically used for spaces created at [thewired.space](https://thewired.space), but you can always change it to a different server, or even run your own. The host server can be set when publishing your space.

The host server acts like a traditional game server - it manages player connections to your space, handles communication between everyone, and acts as as a source of truth for the state of the space.
