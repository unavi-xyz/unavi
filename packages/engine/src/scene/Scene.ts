import { Document, Node } from "@gltf-transform/core";
import { KHRXMP } from "@gltf-transform/extensions";
import {
  AudioExtension,
  AvatarExtension,
  BehaviorExtension,
  ColliderExtension,
  SpaceExtension,
  SPAWN_TITLE,
  SpawnPoint,
  SpawnPointExtension,
} from "@unavi/gltf-extensions";

import { Accessors } from "./attributes/Accessors";
import { Animations } from "./attributes/Animations";
import { Buffers } from "./attributes/Buffers";
import { Materials } from "./attributes/Materials";
import { Meshes } from "./attributes/Meshes";
import { Nodes } from "./attributes/Nodes";
import { Primitives } from "./attributes/Primitives";
import { Skins } from "./attributes/Skins";
import { Textures } from "./attributes/Textures";

/**
 * The internal representation of a scene.
 *
 * @group Scene
 */
export class Scene {
  doc = new Document();

  extensions = {
    audio: this.doc.createExtension(AudioExtension),
    avatar: this.doc.createExtension(AvatarExtension),
    behavior: this.doc.createExtension(BehaviorExtension),
    collider: this.doc.createExtension(ColliderExtension),
    space: this.doc.createExtension(SpaceExtension),
    spawn: this.doc.createExtension(SpawnPointExtension),
    xmp: this.doc.createExtension(KHRXMP),
  };

  buffer = new Buffers(this);
  accessor = new Accessors(this);
  texture = new Textures(this);
  material = new Materials(this);
  primitive = new Primitives(this);
  mesh = new Meshes(this);
  node = new Nodes(this);
  skin = new Skins(this);
  animation = new Animations(this);

  /**
   * Adds the given document to the scene.
   * @param doc The document to merge.
   */
  async addDocument(doc: Document) {
    // Merge documents
    this.doc.merge(doc);

    // Merge into one scene
    const scenes = this.doc.getRoot().listScenes();
    const newScene = this.doc.createScene();
    this.doc.getRoot().setDefaultScene(newScene);

    scenes.forEach((scene) => {
      // Move all nodes to new scene
      scene.listChildren().forEach((node) => newScene.addChild(node));
      scene.dispose();
    });

    this.processChanges();

    // Wait to let threads to finish
    await new Promise((resolve) => setTimeout(resolve, 200));
  }

  processChanges() {
    this.buffer.processChanges();
    this.accessor.processChanges();
    this.texture.processChanges();
    this.material.processChanges();
    this.primitive.processChanges();
    this.mesh.processChanges();
    this.node.processChanges();
    this.skin.processChanges();
    this.animation.processChanges();
  }

  getSpawn(title: (typeof SPAWN_TITLE)[keyof typeof SPAWN_TITLE] = "Default"): Node | undefined {
    return this.doc
      .getRoot()
      .listNodes()
      .find((node) => {
        const spawn = node.getExtension<SpawnPoint>(SpawnPointExtension.EXTENSION_NAME);
        return spawn && spawn.getTitle() === title;
      });
  }
}
