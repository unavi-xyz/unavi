import { Document } from "@gltf-transform/core";

import { ColliderExtension } from "../gltf";
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
}
