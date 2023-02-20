import { Extension, ReaderContext, WriterContext } from "@gltf-transform/core";

import { EXTENSION_NAME } from "./constants";
import { SpawnPoint } from "./SpawnPoint";

type SpawnPointDef = {
  title: string;
};

/**
 * @link https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/OMI_spawn_point
 */
export class SpawnPointExtension extends Extension {
  static override readonly EXTENSION_NAME = EXTENSION_NAME;
  override readonly extensionName = EXTENSION_NAME;

  public createSpawnPoint(): SpawnPoint {
    return new SpawnPoint(this.document.getGraph());
  }

  public read(context: ReaderContext) {
    const nodeDefs = context.jsonDoc.json.nodes || [];

    nodeDefs.forEach((nodeDef, nodeIndex) => {
      if (!nodeDef.extensions || !nodeDef.extensions[EXTENSION_NAME]) return;

      const node = context.nodes[nodeIndex];
      if (!node) throw new Error("Node not found");

      const spawnPoint = this.createSpawnPoint();

      const rootDef = nodeDef.extensions[EXTENSION_NAME] as SpawnPointDef;
      spawnPoint.title = rootDef.title;

      node.setExtension(EXTENSION_NAME, spawnPoint);
    });

    return this;
  }

  public write(context: WriterContext) {
    this.document
      .getRoot()
      .listNodes()
      .forEach((node) => {
        const spawnPoint = node.getExtension<SpawnPoint>(EXTENSION_NAME);

        if (spawnPoint) {
          const nodeIndex = context.nodeIndexMap.get(node);
          if (nodeIndex === undefined) throw new Error("Node index not found");

          const nodes = context.jsonDoc.json.nodes;
          if (!nodes) throw new Error("Nodes not found");

          const nodeDef = nodes[nodeIndex];
          if (!nodeDef) throw new Error("Node def not found");

          nodeDef.extensions = nodeDef.extensions || {};

          const def: SpawnPointDef = { title: spawnPoint.title };
          nodeDef.extensions[EXTENSION_NAME] = def;
        }
      });

    return this;
  }
}
