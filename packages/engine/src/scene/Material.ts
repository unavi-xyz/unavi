import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";

import { Quad, Triplet } from "../types";
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

  colorTextureId$ = new BehaviorSubject<string | null>(null);
  emissiveTextureId$ = new BehaviorSubject<string | null>(null);
  normalTextureId$ = new BehaviorSubject<string | null>(null);
  occlusionTextureId$ = new BehaviorSubject<string | null>(null);
  metallicRoughnessTextureId$ = new BehaviorSubject<string | null>(null);

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

  get colorTextureId() {
    return this.colorTextureId$.value;
  }

  set colorTextureId(colorTextureId: string | null) {
    this.colorTextureId$.next(colorTextureId);
  }

  get emissiveTextureId() {
    return this.emissiveTextureId$.value;
  }

  set emissiveTextureId(emissiveTextureId: string | null) {
    this.emissiveTextureId$.next(emissiveTextureId);
  }

  get normalTextureId() {
    return this.normalTextureId$.value;
  }

  set normalTextureId(normalTextureId: string | null) {
    this.normalTextureId$.next(normalTextureId);
  }

  get occlusionTextureId() {
    return this.occlusionTextureId$.value;
  }

  set occlusionTextureId(occlusionTextureId: string | null) {
    this.occlusionTextureId$.next(occlusionTextureId);
  }

  get metallicRoughnessTextureId() {
    return this.metallicRoughnessTextureId$.value;
  }

  set metallicRoughnessTextureId(metallicRoughnessTextureId: string | null) {
    this.metallicRoughnessTextureId$.next(metallicRoughnessTextureId);
  }

  destroy() {
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
    this.colorTextureId$.complete();
    this.emissiveTextureId$.complete();
    this.normalTextureId$.complete();
    this.occlusionTextureId$.complete();
    this.metallicRoughnessTextureId$.complete();
  }

  toJSON(): MaterialJSON {
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
      colorTextureId: this.colorTextureId,
      emissiveTextureId: this.emissiveTextureId,
      normalTextureId: this.normalTextureId,
      occlusionTextureId: this.occlusionTextureId,
      metallicRoughnessTextureId: this.metallicRoughnessTextureId,
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
    if (json.colorTextureId !== undefined)
      this.colorTextureId = json.colorTextureId;
    if (json.emissiveTextureId !== undefined)
      this.emissiveTextureId = json.emissiveTextureId;
    if (json.normalTextureId !== undefined)
      this.normalTextureId = json.normalTextureId;
    if (json.occlusionTextureId !== undefined)
      this.occlusionTextureId = json.occlusionTextureId;
    if (json.metallicRoughnessTextureId !== undefined)
      this.metallicRoughnessTextureId = json.metallicRoughnessTextureId;
  }

  static fromJSON(json: MaterialJSON) {
    const material = new Material({ id: json.id });
    material.applyJSON(json);
    return material;
  }
}
