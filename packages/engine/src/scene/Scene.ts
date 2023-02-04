import { Document } from "@gltf-transform/core";

import { ColliderExtension, SPAWN_TITLES, SpawnPoint, SpawnPointExtension } from "../gltf";
import { Accessors } from "./attributes/Accessors";
import { Buffers } from "./attributes/Buffers";
import { Materials } from "./attributes/Materials";
import { Meshes } from "./attributes/Meshes";
import { Nodes } from "./attributes/Nodes";
import { Primitives } from "./attributes/Primitives";
import { Textures } from "./attributes/Textures";

export class Scene {
  doc = new Document();

  extensions = {
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

  loadDocument(doc: Document) {
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

  getSpawn(title: (typeof SPAWN_TITLES)[keyof typeof SPAWN_TITLES] = "Default") {
    return this.doc
      .getRoot()
      .listNodes()
      .find((node) => {
        const spawn = node.getExtension<SpawnPoint>(SpawnPointExtension.EXTENSION_NAME);
        return spawn && spawn.title === title;
      });
  }
}
