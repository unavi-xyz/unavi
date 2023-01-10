import { Document, Node } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Vec3, Vec4 } from "../../types";
import { Utils } from "./types";

type NodeId = string;

export interface NodeJSON {
  name: string;
  translation: Vec3;
  rotation: Vec4;
  scale: Vec3;
  children: NodeId[];
}

export class NodeUtils implements Utils<Node, NodeJSON> {
  #doc: Document;

  store = new Map<string, Node>();

  constructor(doc: Document) {
    this.#doc = doc;
  }

  getId(node: Node) {
    for (const [id, n] of this.store) {
      if (n === node) return id;
    }
  }

  create(json: Partial<NodeJSON> = {}, id?: string) {
    const node = this.#doc.createNode();
    this.applyJSON(node, json);

    const { id: nodeId } = this.process(node, id);

    return { id: nodeId, object: node };
  }

  process(node: Node, id?: string) {
    const nodeId = id ?? nanoid();
    this.store.set(nodeId, node);

    node.addEventListener("dispose", () => {
      this.store.delete(nodeId);
    });

    return { id: nodeId };
  }

  processChanges() {
    const changed: Node[] = [];

    // Add new nodes
    this.#doc
      .getRoot()
      .listNodes()
      .forEach((node) => {
        const nodeId = this.getId(node);
        if (nodeId) return;

        this.process(node);
        changed.push(node);
      });

    return changed;
  }

  applyJSON(node: Node, json: Partial<NodeJSON>) {
    if (json.name !== undefined) node.setName(json.name);
    if (json.translation) node.setTranslation(json.translation);
    if (json.rotation) node.setRotation(json.rotation);
    if (json.scale) node.setScale(json.scale);

    if (json.children) {
      for (const childId of json.children) {
        const child = this.store.get(childId);
        if (!child) continue;

        node.addChild(child);
      }
    }
  }

  toJSON(node: Node): NodeJSON {
    const childrenIds = node.listChildren().map((child) => {
      for (const [id, node] of this.store) {
        if (node === child) return id;
      }

      throw new Error("Node not found");
    });

    return {
      name: node.getName(),
      translation: node.getTranslation(),
      rotation: node.getRotation(),
      scale: node.getScale(),
      children: childrenIds,
    };
  }
}
