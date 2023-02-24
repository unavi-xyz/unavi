import { Document } from "@gltf-transform/core";

import {
  BehaviorExtension,
  ColliderExtension,
  SPAWN_TITLE,
  SpawnPoint,
  SpawnPointExtension,
} from "../gltf";
import { Accessors } from "./attributes/Accessors";
import { Buffers } from "./attributes/Buffers";
import { Materials } from "./attributes/Materials";
import { Meshes } from "./attributes/Meshes";
import { Nodes } from "./attributes/Nodes";
import { Primitives } from "./attributes/Primitives";
import { Textures } from "./attributes/Textures";

/**
 * The internal representation of a scene.
 *
 * @group Scene
 */
export class Scene {
  doc = new Document();

  extensions = {
    behavior: this.doc.createExtension(BehaviorExtension),
    collider: this.doc.createExtension(ColliderExtension),
    spawn: this.doc.createExtension(SpawnPointExtension),
  };

  buffer = new Buffers(this.doc);
  accessor = new Accessors(this.doc, this.buffer);
  texture = new Textures(this.doc);
  material = new Materials(this.doc, this.texture);
  primitive = new Primitives(this.doc, this.accessor, this.material);
  mesh = new Meshes(this.doc, this.primitive);
  node = new Nodes(this);

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
  }

  getSpawn(title: (typeof SPAWN_TITLE)[keyof typeof SPAWN_TITLE] = "Default") {
    return this.doc
      .getRoot()
      .listNodes()
      .find((node) => {
        const spawn = node.getExtension<SpawnPoint>(SpawnPointExtension.EXTENSION_NAME);
        return spawn && spawn.title === title;
      });
  }
}
