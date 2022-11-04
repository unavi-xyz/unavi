import { BaseMesh } from "./BaseMesh";
import { Primitive } from "./Primitive";
import { PrimitivesMeshJSON } from "./types";

export class PrimitivesMesh extends BaseMesh {
  readonly type = "Primitives";

  primitives: Primitive[] = [];

  constructor(id?: string) {
    super(id);
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string | null) {
    this.name$.next(name);
  }

  get materialId() {
    return this.materialId$.value;
  }

  set materialId(materialId: string | null) {
    this.materialId$.next(materialId);
  }

  destroy() {
    this.name$.complete();
    this.materialId$.complete();
  }

  toJSON(): PrimitivesMeshJSON {
    const primitives = this.primitives.map((primitive) => primitive.toJSON());
    return {
      type: this.type,
      isInternal: this.isInternal,
      id: this.id,
      name: this.name,
      materialId: null,
      primitives,
    };
  }

  applyJSON(json: Partial<PrimitivesMeshJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
    if (json.materialId !== undefined) this.materialId = json.materialId;
    if (json.primitives !== undefined)
      this.primitives = json.primitives.map((primitiveJSON) => {
        const primitive = new Primitive();
        primitive.applyJSON(primitiveJSON);
        return primitive;
      });
  }

  static fromJSON(json: PrimitivesMeshJSON) {
    const mesh = new PrimitivesMesh(json.id);
    mesh.applyJSON(json);
    return mesh;
  }
}
