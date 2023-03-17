import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "../constants";
import { SpawnPoint } from "./SpawnPoint";

type SpawnPointDef = {
  title: string;
};

/**
 * Implementation of the {@link https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/OMI_spawn_point OMI_spawn_point} extension.
 *
 * @group GLTF Extensions
 */
export class SpawnPointExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME.SpawnPoint;
  override readonly extensionName = EXTENSION_NAME.SpawnPoint;

  public createSpawnPoint(): SpawnPoint {
    return new SpawnPoint(this.document.getGraph());
  }

  public read(context: ReaderContext) {
    const nodeDefs = context.jsonDoc.json.nodes || [];

    nodeDefs.forEach((nodeDef, nodeIndex) => {
      if (!nodeDef.extensions || !nodeDef.extensions[this.extensionName]) return;

      const node = context.nodes[nodeIndex];
      if (!node) throw new Error("Node not found");

      const spawnPoint = this.createSpawnPoint();

      const rootDef = nodeDef.extensions[this.extensionName] as SpawnPointDef;
      spawnPoint.setTitle(rootDef.title);

      node.setExtension(this.extensionName, spawnPoint);
    });

    return this;
  }

  public write(context: WriterContext) {
    this.document
      .getRoot()
      .listNodes()
      .forEach((node) => {
        const spawnPoint = node.getExtension<SpawnPoint>(this.extensionName);

        if (spawnPoint) {
          const nodeIndex = context.nodeIndexMap.get(node);
          if (nodeIndex === undefined) throw new Error("Node index not found");

          const nodes = context.jsonDoc.json.nodes;
          if (!nodes) throw new Error("Nodes not found");

          const nodeDef = nodes[nodeIndex];
          if (!nodeDef) throw new Error("Node def not found");

          nodeDef.extensions = nodeDef.extensions || {};

          const def: SpawnPointDef = { title: spawnPoint.getTitle() };
          nodeDef.extensions[this.extensionName] = def;
        }
      });

    return this;
  }
}
