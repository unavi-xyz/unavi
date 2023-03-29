---
sidebar_position: 10
sidebar_label: 🌏 Spaces
title: Spaces
---

Spaces are 3D multiplayer environments that serve as the core building blocks of the platform.

## 🏗️ Creation

Spaces can be created for free using the [Editor](https://www.thewired.space/create).

<div class="large-img">
  <img src="/img/Editor.png" />
</div>

Space scenes are stored internally as a single [glTF](https://github.com/KhronosGroup/glTF) file, an open file format supported by various engines and tools like Blender. Within the editor, you can import glTF models and add game logic, such as physics colliders or spawn locations (stored as [glTF extensions](https://github.com/KhronosGroup/glTF/blob/main/extensions/README.md)). A node-based scripting system can also be used to add dynamic and interactive behaviors to your scene.

## 🏙️ Publishing

Once your space is ready, you can publish it to the blockchain as an NFT. The blockchain is used as a decentralized index of all spaces within The Wired. Once you publish your space it becomes available to the world.

## ☁️ Hosting

Just like websites, spaces need servers to host them.

The host server adds **multiplayer functionality** to your space. It acts as an [SFU](https://bloggeek.me/webrtcglossary/sfu/) for communication between clients, and is responsible for managing the state of your space.

The team runs a free host server that is used by default for any spaces created at [thewired.space](https://thewired.space). However, you also have the option to run your own host server, giving you complete control and ownership of your creations. Check out the [deployment guide](/deployment/host) for more information on how to self-host.

Travel between spaces is not limited to a single host server. You can travel between spaces hosted on different servers, just like you can visit different websites on the web. The ability to self-host spaces is a critical aspect of maintaining The Wired as an open network.
