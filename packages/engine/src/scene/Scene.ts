import { Document } from "@gltf-transform/core";

import {
  BehaviorNode,
  ColliderExtension,
  SPAWN_TITLES,
  SpawnPoint,
  SpawnPointExtension,
} from "../gltf";
import { BehaviorExtension } from "../gltf/extensions/Behavior/BehaviorExtension";
import { Accessors } from "./attributes/Accessors";
import { Buffers } from "./attributes/Buffers";
import { Materials } from "./attributes/Materials";
import { Meshes } from "./attributes/Meshes";
import { Nodes } from "./attributes/Nodes";
import { Primitives } from "./attributes/Primitives";
import { Textures } from "./attributes/Textures";

/**
 * The internal representation of a scene.
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
    this.doc.merge(doc);

    // Transfer behavior nodes
    const behaviorNodes = doc
      .getRoot()
      .listExtensionsUsed()
      .filter((ext) => ext instanceof BehaviorExtension)
      .flatMap((ext) =>
        ext.listProperties().filter((p): p is BehaviorNode => p instanceof BehaviorNode)
      );

    const newBehaviorNodes = behaviorNodes.map((behaviorNode) => {
      const newBehaviorNode = this.extensions.behavior.createBehaviorNode();
      newBehaviorNode.name = behaviorNode.name;
      newBehaviorNode.type = behaviorNode.type;
      newBehaviorNode.parameters = behaviorNode.parameters;
      newBehaviorNode.flow = behaviorNode.flow;
      newBehaviorNode.setExtras(behaviorNode.getExtras());
      return newBehaviorNode;
    });

    newBehaviorNodes.forEach((behaviorNode) => {
      if (behaviorNode.parameters) {
        Object.entries(behaviorNode.parameters).forEach(([key, value]) => {
          if (typeof value === "object" && "link" in value) {
            const newNode = newBehaviorNodes.find((node) => node.name === value.link.name);
            if (!newNode) throw new Error("Invalid behavior node reference");

            if (!behaviorNode.parameters) behaviorNode.parameters = {};
            behaviorNode.parameters[key] = { link: newNode, socket: value.socket };
          }
        });
      }

      if (behaviorNode.flow) {
        Object.entries(behaviorNode.flow).forEach(([key, value]) => {
          const newNode = newBehaviorNodes.find((node) => node.name === value.name);
          if (!newNode) throw new Error("Invalid behavior node reference");

          if (!behaviorNode.flow) behaviorNode.flow = {};
          behaviorNode.flow[key] = newNode;
        });
      }
    });

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
