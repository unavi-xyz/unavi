import { WebIO } from "@gltf-transform/core";

import { RenderModule } from "../render/RenderModule";
import { Scene } from "./Scene";
import { MaterialJSON } from "./utils/MaterialUtils";
import { NodeJSON } from "./utils/NodeUtils";
import { PrimitiveJSON } from "./utils/PrimitiveUtils";

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

      buffer.addEventListener("dispose", () => {
        this.#render.toRenderThread({
          subject: "dispose_buffer",
          data: id,
        });
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

      accessor.addEventListener("dispose", () => {
        this.#render.toRenderThread({
          subject: "dispose_accessor",
          data: id,
        });
      });
    });

    this.texture.processChanges().forEach((texture) => {
      const id = this.texture.getId(texture);
      if (!id) throw new Error("Id not found");
      const json = this.texture.toJSON(texture);
      const transferable = json.image ? [json.image.buffer] : [];

      this.#render.toRenderThread(
        {
          subject: "create_texture",
          data: { id, json },
        },
        transferable
      );

      texture.addEventListener("dispose", () => {
        this.#render.toRenderThread({
          subject: "dispose_texture",
          data: id,
        });
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

      material.addEventListener("change", (e) => {
        const attribute = e.attribute as keyof MaterialJSON;
        const json = this.material.toJSON(material);
        const value = json[attribute];

        this.#render.toRenderThread({
          subject: "change_material",
          data: { id, json: { [attribute]: value } },
        });
      });

      material.addEventListener("dispose", () => {
        this.#render.toRenderThread({
          subject: "dispose_material",
          data: id,
        });
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

      primitive.addEventListener("change", (e) => {
        const attribute = e.attribute as keyof PrimitiveJSON;
        const json = this.primitive.toJSON(primitive);
        const value = json[attribute];

        this.#render.toRenderThread({
          subject: "change_primitive",
          data: { id, json: { [attribute]: value } },
        });
      });

      primitive.addEventListener("dispose", () => {
        this.#render.toRenderThread({
          subject: "dispose_primitive",
          data: id,
        });
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

      mesh.addEventListener("dispose", () => {
        this.#render.toRenderThread({
          subject: "dispose_mesh",
          data: id,
        });
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

      node.addEventListener("change", (e) => {
        const attribute = e.attribute as keyof NodeJSON;
        const json = this.node.toJSON(node);
        const value = json[attribute];

        this.#render.toRenderThread({
          subject: "change_node",
          data: { id, json: { [attribute]: value } },
        });
      });

      node.addEventListener("dispose", () => {
        this.#render.toRenderThread({
          subject: "dispose_node",
          data: id,
        });
      });
    });
  }
}
