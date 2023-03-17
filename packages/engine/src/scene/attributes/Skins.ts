import { Document, Skin } from "@gltf-transform/core";
import { nanoid } from "nanoid";

import { Scene } from "../Scene";
import { Accessors } from "./Accessors";
import { Attribute } from "./Attribute";
import { Nodes } from "./Nodes";

export interface SkinJSON {
  jointIds: string[];
  inverseBindMatricesId: string | null;
  skeletonId: string | null;
}

/**
 * Stores and manages skins within a {@link Scene}.
 *
 * @group Scene
 */
export class Skins extends Attribute<Skin, SkinJSON> {
  #doc: Document;
  #node: Nodes;
  #accessor: Accessors;

  store = new Map<string, Skin>();

  constructor(scene: Scene) {
    super();

    this.#doc = scene.doc;
    this.#node = scene.node;
    this.#accessor = scene.accessor;
  }

  getId(skin: Skin) {
    for (const [id, m] of this.store) {
      if (m === skin) return id;
    }
  }

  create(json: Partial<SkinJSON> = {}, id?: string) {
    const skin = this.#doc.createSkin();
    this.applyJSON(skin, json);

    const { id: skinId } = this.process(skin, id);

    this.emitCreate(skinId);

    return { id: skinId, object: skin };
  }

  process(skin: Skin, id?: string) {
    const skinId = id ?? nanoid();
    this.store.set(skinId, skin);

    skin.addEventListener("dispose", () => {
      this.store.delete(skinId);
    });

    return { id: skinId };
  }

  processChanges() {
    const changed: Skin[] = [];

    // Add new skins
    this.#doc
      .getRoot()
      .listSkins()
      .forEach((skin) => {
        const skinId = this.getId(skin);
        if (skinId) return;

        this.process(skin);
        changed.push(skin);
      });

    return changed;
  }

  applyJSON(skin: Skin, json: Partial<SkinJSON>) {
    const { jointIds, inverseBindMatricesId, skeletonId } = json;

    if (jointIds !== undefined) {
      skin.listJoints().forEach((joint) => skin.removeJoint(joint));

      jointIds.forEach((jointId) => {
        const joint = this.#node.store.get(jointId);
        if (joint) skin.addJoint(joint);
      });
    }

    if (inverseBindMatricesId !== undefined) {
      if (inverseBindMatricesId === null) {
        skin.setInverseBindMatrices(null);
      } else {
        const inverseBindMatrices = this.#accessor.store.get(inverseBindMatricesId);
        if (inverseBindMatrices) skin.setInverseBindMatrices(inverseBindMatrices);
      }
    }

    if (skeletonId !== undefined) {
      if (skeletonId === null) {
        skin.setSkeleton(null);
      } else {
        const skeleton = this.#node.store.get(skeletonId);
        if (skeleton) skin.setSkeleton(skeleton);
      }
    }
  }

  toJSON(skin: Skin): SkinJSON {
    const jointIds = skin
      .listJoints()
      .map((joint) => this.#node.getId(joint))
      .filter((id): id is string => Boolean(id));

    const inverseBindMatrices = skin.getInverseBindMatrices();
    const inverseBindMatricesId = inverseBindMatrices
      ? this.#accessor.getId(inverseBindMatrices) ?? null
      : null;

    const skeleton = skin.getSkeleton();
    const skeletonId = skeleton ? this.#node.getId(skeleton) ?? null : null;

    return {
      jointIds,
      inverseBindMatricesId,
      skeletonId,
    };
  }
}
