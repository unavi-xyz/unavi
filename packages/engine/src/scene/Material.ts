import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { Quad, Triplet } from "../types";
import { Texture } from "./Texture";
import { MaterialJSON } from "./types";

export class Material {
  readonly id: string;

  name$ = new BehaviorSubject("New Material");
  isInternal$ = new BehaviorSubject(false);

  alpha$ = new BehaviorSubject(1);
  alphaCutoff$ = new BehaviorSubject(0.5);
  alphaMode$ = new BehaviorSubject<"OPAQUE" | "MASK" | "BLEND">("OPAQUE");

  doubleSided$ = new BehaviorSubject(false);
  color$ = new BehaviorSubject<Quad>([1, 1, 1, 1]);
  emissive$ = new BehaviorSubject<Triplet>([0, 0, 0]);
  normalScale$ = new BehaviorSubject(1);
  occlusionStrength$ = new BehaviorSubject(1);
  roughness$ = new BehaviorSubject(1);
  metalness$ = new BehaviorSubject(0);

  colorTexture$ = new BehaviorSubject<Texture | null>(null);
  emissiveTexture$ = new BehaviorSubject<Texture | null>(null);
  normalTexture$ = new BehaviorSubject<Texture | null>(null);
  occlusionTexture$ = new BehaviorSubject<Texture | null>(null);
  metallicRoughnessTexture$ = new BehaviorSubject<Texture | null>(null);

  constructor({ id = nanoid() }: { id?: string } = {}) {
    this.id = id;
  }

  get name() {
    return this.name$.value;
  }

  set name(name: string) {
    this.name$.next(name);
  }

  get isInternal() {
    return this.isInternal$.value;
  }

  set isInternal(isInternal: boolean) {
    this.isInternal$.next(isInternal);
  }

  get doubleSided() {
    return this.doubleSided$.value;
  }

  set doubleSided(doubleSided: boolean) {
    this.doubleSided$.next(doubleSided);
  }

  get color() {
    return this.color$.value;
  }

  set color(color: Quad) {
    const clamped: Quad = [
      Math.max(0, Math.min(1, color[0])),
      Math.max(0, Math.min(1, color[1])),
      Math.max(0, Math.min(1, color[2])),
      Math.max(0, Math.min(1, color[3])),
    ];
    this.color$.next(clamped);
  }

  get emissive() {
    return this.emissive$.value;
  }

  set emissive(emissive: Triplet) {
    const clamped: Triplet = [
      Math.max(0, Math.min(1, emissive[0])),
      Math.max(0, Math.min(1, emissive[1])),
      Math.max(0, Math.min(1, emissive[2])),
    ];
    this.emissive$.next(clamped);
  }

  get roughness() {
    return this.roughness$.value;
  }

  set roughness(roughness: number) {
    const clamped = Math.max(0, Math.min(1, roughness));
    this.roughness$.next(clamped);
  }

  get metalness() {
    return this.metalness$.value;
  }

  set metalness(metalness: number) {
    const clamped = Math.max(0, Math.min(1, metalness));
    this.metalness$.next(clamped);
  }

  get alpha() {
    return this.alpha$.value;
  }

  set alpha(alpha: number) {
    const clamped = Math.max(0, Math.min(1, alpha));
    this.alpha$.next(clamped);
  }

  get alphaCutoff() {
    return this.alphaCutoff$.value;
  }

  set alphaCutoff(alphaCutoff: number) {
    const clamped = Math.max(0, Math.min(1, alphaCutoff));
    this.alphaCutoff$.next(clamped);
  }

  get alphaMode() {
    return this.alphaMode$.value;
  }

  set alphaMode(alphaMode: "OPAQUE" | "MASK" | "BLEND") {
    this.alphaMode$.next(alphaMode);
  }

  get normalScale() {
    return this.normalScale$.value;
  }

  set normalScale(normalScale: number) {
    const clamped = Math.max(0, Math.min(1, normalScale));
    this.normalScale$.next(clamped);
  }

  get occlusionStrength() {
    return this.occlusionStrength$.value;
  }

  set occlusionStrength(occlusionStrength: number) {
    const clamped = Math.max(0, Math.min(1, occlusionStrength));
    this.occlusionStrength$.next(clamped);
  }

  get colorTexture() {
    return this.colorTexture$.value;
  }

  set colorTexture(colorTexture: Texture | null) {
    this.colorTexture$.next(colorTexture);
  }

  get emissiveTexture() {
    return this.emissiveTexture$.value;
  }

  set emissiveTexture(emissiveTexture: Texture | null) {
    this.emissiveTexture$.next(emissiveTexture);
  }

  get normalTexture() {
    return this.normalTexture$.value;
  }

  set normalTexture(normalTexture: Texture | null) {
    this.normalTexture$.next(normalTexture);
  }

  get occlusionTexture() {
    return this.occlusionTexture$.value;
  }

  set occlusionTexture(occlusionTexture: Texture | null) {
    this.occlusionTexture$.next(occlusionTexture);
  }

  get metallicRoughnessTexture() {
    return this.metallicRoughnessTexture$.value;
  }

  set metallicRoughnessTexture(metallicRoughnessTexture: Texture | null) {
    this.metallicRoughnessTexture$.next(metallicRoughnessTexture);
  }

  destroy() {
    this.colorTexture?.destroy();
    this.emissiveTexture?.destroy();
    this.normalTexture?.destroy();
    this.occlusionTexture?.destroy();
    this.metallicRoughnessTexture?.destroy();

    this.name$.complete();
    this.isInternal$.complete();
    this.alpha$.complete();
    this.alphaCutoff$.complete();
    this.alphaMode$.complete();
    this.doubleSided$.complete();
    this.color$.complete();
    this.emissive$.complete();
    this.normalScale$.complete();
    this.occlusionStrength$.complete();
    this.roughness$.complete();
    this.metalness$.complete();
    this.colorTexture$.complete();
    this.emissiveTexture$.complete();
    this.normalTexture$.complete();
    this.occlusionTexture$.complete();
    this.metallicRoughnessTexture$.complete();
  }

  toJSON(): MaterialJSON {
    const colorTexture = this.colorTexture ? this.colorTexture.toJSON() : null;
    const emissiveTexture = this.emissiveTexture
      ? this.emissiveTexture.toJSON()
      : null;
    const normalTexture = this.normalTexture
      ? this.normalTexture.toJSON()
      : null;
    const occlusionTexture = this.occlusionTexture
      ? this.occlusionTexture.toJSON()
      : null;
    const metallicRoughnessTexture = this.metallicRoughnessTexture
      ? this.metallicRoughnessTexture.toJSON()
      : null;

    return {
      id: this.id,
      name: this.name,
      isInternal: this.isInternal,
      doubleSided: this.doubleSided,
      color: this.color,
      emissive: this.emissive,
      roughness: this.roughness,
      metalness: this.metalness,
      alpha: this.alpha,
      alphaCutoff: this.alphaCutoff,
      alphaMode: this.alphaMode,
      normalScale: this.normalScale,
      occlusionStrength: this.occlusionStrength,
      colorTexture,
      emissiveTexture,
      normalTexture,
      occlusionTexture,
      metallicRoughnessTexture,
    };
  }

  applyJSON(json: Partial<MaterialJSON>) {
    if (json.name !== undefined) this.name = json.name;
    if (json.isInternal !== undefined) this.isInternal = json.isInternal;
    if (json.doubleSided !== undefined) this.doubleSided = json.doubleSided;
    if (json.color !== undefined) this.color = json.color;
    if (json.emissive !== undefined) this.emissive = json.emissive;
    if (json.roughness !== undefined) this.roughness = json.roughness;
    if (json.metalness !== undefined) this.metalness = json.metalness;
    if (json.alpha !== undefined) this.alpha = json.alpha;
    if (json.alphaCutoff !== undefined) this.alphaCutoff = json.alphaCutoff;
    if (json.alphaMode !== undefined) this.alphaMode = json.alphaMode;
    if (json.normalScale !== undefined) this.normalScale = json.normalScale;
    if (json.occlusionStrength !== undefined)
      this.occlusionStrength = json.occlusionStrength;
    if (json.colorTexture !== undefined)
      this.colorTexture = json.colorTexture
        ? Texture.fromJSON(json.colorTexture)
        : null;
    if (json.emissiveTexture !== undefined)
      this.emissiveTexture = json.emissiveTexture
        ? Texture.fromJSON(json.emissiveTexture)
        : null;
    if (json.normalTexture !== undefined)
      this.normalTexture = json.normalTexture
        ? Texture.fromJSON(json.normalTexture)
        : null;
    if (json.occlusionTexture !== undefined)
      this.occlusionTexture = json.occlusionTexture
        ? Texture.fromJSON(json.occlusionTexture)
        : null;
    if (json.metallicRoughnessTexture !== undefined)
      this.metallicRoughnessTexture = json.metallicRoughnessTexture
        ? Texture.fromJSON(json.metallicRoughnessTexture)
        : null;
  }

  static fromJSON(json: MaterialJSON) {
    const material = new Material({ id: json.id });
    material.applyJSON(json);
    return material;
  }
}
