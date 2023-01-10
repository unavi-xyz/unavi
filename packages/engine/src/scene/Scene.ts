import { Document } from "@gltf-transform/core";

import { AccessorUtils } from "./utils/AccessorUtils";
import { BufferUtils } from "./utils/BufferUtils";
import { MaterialUtils } from "./utils/MaterialUtils";
import { MeshUtils } from "./utils/MeshUtils";
import { NodeUtils } from "./utils/NodeUtils";
import { PrimitiveUtils } from "./utils/PrimitiveUtils";
import { TextureUtils } from "./utils/TextureUtils";

export class Scene {
  doc = new Document();

  buffer = new BufferUtils(this.doc);
  accessor = new AccessorUtils(this.doc, this.buffer);
  texture = new TextureUtils(this.doc);
  material = new MaterialUtils(this.doc, this.texture);
  primitive = new PrimitiveUtils(this.doc, this.accessor, this.material);
  mesh = new MeshUtils(this.doc, this.primitive);
  node = new NodeUtils(this.doc, this.mesh);

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
