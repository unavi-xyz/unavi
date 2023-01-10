import { WebIO } from "@gltf-transform/core";

import { RenderModule } from "../render/RenderModule";
import { Scene } from "./Scene";

export class SceneModule extends Scene {
  #render: RenderModule;

  constructor(render: RenderModule) {
    super();
    this.#render = render;
  }

  async load(uri: string) {
    const io = new WebIO();
    const doc = await io.read(uri);
    this.loadDocument(doc);
  }

  override processChanges() {
    this.buffer.processChanges().forEach((buffer) => {
      const id = this.buffer.getId(buffer);
      if (!id) throw new Error("Id not found");
      const json = this.buffer.toJSON(buffer);

      this.#render.toRenderThread({
        subject: "create_buffer",
        data: { id, json },
      });
    });

    this.accessor.processChanges().forEach((accessor) => {
      const id = this.accessor.getId(accessor);
      if (!id) throw new Error("Id not found");
      const json = this.accessor.toJSON(accessor);

      this.#render.toRenderThread({
        subject: "create_accessor",
        data: { id, json },
      });
    });

    this.texture.processChanges().forEach((texture) => {
      const id = this.texture.getId(texture);
      if (!id) throw new Error("Id not found");
      const json = this.texture.toJSON(texture);

      this.#render.toRenderThread({
        subject: "create_texture",
        data: { id, json },
      });
    });

    this.material.processChanges().forEach((material) => {
      const id = this.material.getId(material);
      if (!id) throw new Error("Id not found");
      const json = this.material.toJSON(material);

      this.#render.toRenderThread({
        subject: "create_material",
        data: { id, json },
      });
    });

    this.primitive.processChanges().forEach((primitive) => {
      const id = this.primitive.getId(primitive);
      if (!id) throw new Error("Id not found");
      const json = this.primitive.toJSON(primitive);

      this.#render.toRenderThread({
        subject: "create_primitive",
        data: { id, json },
      });
    });

    this.mesh.processChanges().forEach((mesh) => {
      const id = this.mesh.getId(mesh);
      if (!id) throw new Error("Id not found");
      const json = this.mesh.toJSON(mesh);

      this.#render.toRenderThread({
        subject: "create_mesh",
        data: { id, json },
      });
    });

    this.node.processChanges().forEach((node) => {
      const id = this.node.getId(node);
      if (!id) throw new Error("Id not found");
      const json = this.node.toJSON(node);

      this.#render.toRenderThread({
        subject: "create_node",
        data: { id, json },
      });
    });
  }
}
