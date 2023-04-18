---
sidebar_position: 10
sidebar_label: 🌏 Spaces
title: Spaces
---

Spaces are 3D multiplayer environments that serve as the core building block of the platform.

## 🏗️ Creation

Spaces can be created for free using the [Editor](https://www.unavi.xyz/create).

<div class="large-img">
  <img src="/img/Editor.png" />
</div>

Space scenes are stored internally as a [glTF](https://github.com/KhronosGroup/glTF) file - an open file format supported by various engines and tools like Blender. Within the editor, you can import glTF models and edit them to add game logic, such as physics colliders or spawn locations (stored as [glTF extensions](https://github.com/KhronosGroup/glTF/blob/main/extensions/README.md)). A node-based scripting system can also be used to add dynamic and interactive behaviors to your scene.

Ultimately, a space is just a glTF file with some additional metadata.

Once your space is ready, you can publish it from the editor to make it available to the public. You can continue to push updates to the space after you publish it. The editor also provides a preview mode that allows you to test your space before publishing it.

## 🏙️ Publishing

Since a space is just a glTF file, all you need to join a space is that file (or a URL pointing to it). You can share your space with others manually by sending them a URL to it, where they can then enter it into their client, or you can use one of the built-in discovery methods.

### Local Discovery

The easiest way to share your space is to publish it locally to your client. This is what happens when you publish a space from the editor - the space is made available to anyone using the same client as you.

However, if someone is using a different client, they will not have access to the database and will not be able to discover your space. This is where NFTs come in.

### NFTs

After you publish a space, you have the option of minting it as an NFT.

Minting a space is beneficial for discovery, as the blockchain can be used as a public index of all spaces that can easily be searched and browsed by others. Minting a space gives people on other clients the ability to discover your space.

:::info
While spaces that are not minted can not be discovered by people on other clients, they can still be accessed by anyone with the space's URL. You can always share your space with others manually, without minting an NFT.
:::

In the future, we plan to add additional functionality to spaces that are minted as NFTs, such as the ability to monetize your space.

## ☁️ Hosting

Just like websites, spaces need servers to host them.

The host server adds **multiplayer functionality** to your space. It acts as an [SFU](https://bloggeek.me/webrtcglossary/sfu/) for communication between clients, and is responsible for managing the state of your space.

The team runs a free host server that is used by default for any spaces created at [unavi.xyz](https://unavi.xyz). However, you also have the option to run your own host server, giving you complete control and ownership of your creations. Check out the [deployment guide](/deployment/host) for more information on how to self-host.

Travel between spaces is not limited to a single host server. You can travel between spaces hosted on different servers, just like you can visit different websites on the web. The ability to self-host spaces is a critical aspect of maintaining The Wired as an open network.
