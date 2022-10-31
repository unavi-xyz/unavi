import { GLTF } from "@gltf-transform/core";
import { BehaviorSubject } from "rxjs";

import { BaseMesh } from "./BaseMesh";
import { PrimitiveJSON } from "./types";

export class Primitive extends BaseMesh {
  gltfId: string | null = null;

  mode$ = new BehaviorSubject<GLTF.MeshPrimitiveMode>(4);
  indicesId$ = new BehaviorSubject<string | null>(null);

  weights$ = new BehaviorSubject<number[]>([]);
  morphPositionIds$ = new BehaviorSubject<string[]>([]);
  morphNormalIds$ = new BehaviorSubject<string[]>([]);
  morphTangentIds$ = new BehaviorSubject<string[]>([]);

  POSITION$ = new BehaviorSubject<string | null>(null);
  NORMAL$ = new BehaviorSubject<string | null>(null);
  TANGENT$ = new BehaviorSubject<string | null>(null);
  TEXCOORD_0$ = new BehaviorSubject<string | null>(null);
  TEXCOORD_1$ = new BehaviorSubject<string | null>(null);
  COLOR_0$ = new BehaviorSubject<string | null>(null);
  JOINTS_0$ = new BehaviorSubject<string | null>(null);
  WEIGHTS_0$ = new BehaviorSubject<string | null>(null);

  skin: null | {
    inverseBindMatricesId: string;
    jointIds: string[];
  } = null;

  constructor(id?: string) {
    super(id);
  }

  get mode() {
    return this.mode$.value;
  }

  set mode(mode: GLTF.MeshPrimitiveMode) {
    this.mode$.next(mode);
  }

  get indicesId() {
    return this.indicesId$.value;
  }

  set indicesId(indicesId: string | null) {
    this.indicesId$.next(indicesId);
  }

  get weights() {
    return this.weights$.value;
  }

  set weights(weights: number[]) {
    this.weights$.next(weights);
  }

  get morphPositionIds() {
    return this.morphPositionIds$.value;
  }

  set morphPositionIds(morphPositionIds: string[]) {
    this.morphPositionIds$.next(morphPositionIds);
  }

  get morphNormalIds() {
    return this.morphNormalIds$.value;
  }

  set morphNormalIds(morphNormalIds: string[]) {
    this.morphNormalIds$.next(morphNormalIds);
  }

  get morphTangentIds() {
    return this.morphTangentIds$.value;
  }

  set morphTangentIds(morphTangentIds: string[]) {
    this.morphTangentIds$.next(morphTangentIds);
  }

  get POSITION() {
    return this.POSITION$.value;
  }

  set POSITION(POSITION: string | null) {
    this.POSITION$.next(POSITION);
  }

  get NORMAL() {
    return this.NORMAL$.value;
  }

  set NORMAL(NORMAL: string | null) {
    this.NORMAL$.next(NORMAL);
  }

  get TANGENT() {
    return this.TANGENT$.value;
  }

  set TANGENT(TANGENT: string | null) {
    this.TANGENT$.next(TANGENT);
  }

  get TEXCOORD_0() {
    return this.TEXCOORD_0$.value;
  }

  set TEXCOORD_0(TEXCOORD_0: string | null) {
    this.TEXCOORD_0$.next(TEXCOORD_0);
  }

  get TEXCOORD_1() {
    return this.TEXCOORD_1$.value;
  }

  set TEXCOORD_1(TEXCOORD_1: string | null) {
    this.TEXCOORD_1$.next(TEXCOORD_1);
  }

  get COLOR_0() {
    return this.COLOR_0$.value;
  }

  set COLOR_0(COLOR_0: string | null) {
    this.COLOR_0$.next(COLOR_0);
  }

  get JOINTS_0() {
    return this.JOINTS_0$.value;
  }

  set JOINTS_0(JOINTS_0: string | null) {
    this.JOINTS_0$.next(JOINTS_0);
  }

  get WEIGHTS_0() {
    return this.WEIGHTS_0$.value;
  }

  set WEIGHTS_0(WEIGHTS_0: string | null) {
    this.WEIGHTS_0$.next(WEIGHTS_0);
  }

  destroy() {
    this.materialId$.complete();
    this.mode$.complete();
    this.indicesId$.complete();
    this.weights$.complete();
    this.morphPositionIds$.complete();
    this.morphNormalIds$.complete();
    this.morphTangentIds$.complete();
    this.POSITION$.complete();
    this.NORMAL$.complete();
    this.TANGENT$.complete();
    this.TEXCOORD_0$.complete();
    this.TEXCOORD_1$.complete();
    this.COLOR_0$.complete();
    this.JOINTS_0$.complete();
    this.WEIGHTS_0$.complete();
  }

  toJSON(): PrimitiveJSON {
    return {
      id: this.id,
      materialId: this.materialId,
      gltfId: this.gltfId,
      mode: this.mode,
      indicesId: this.indicesId,
      weights: this.weights,
      morphPositionIds: this.morphPositionIds,
      morphNormalIds: this.morphNormalIds,
      morphTangentIds: this.morphTangentIds,
      POSITION: this.POSITION,
      NORMAL: this.NORMAL,
      TANGENT: this.TANGENT,
      TEXCOORD_0: this.TEXCOORD_0,
      TEXCOORD_1: this.TEXCOORD_1,
      COLOR_0: this.COLOR_0,
      JOINTS_0: this.JOINTS_0,
      WEIGHTS_0: this.WEIGHTS_0,
      skin: this.skin,
    };
  }

  applyJSON(json: Partial<PrimitiveJSON>) {
    if (json.materialId !== undefined) this.materialId = json.materialId;
    if (json.gltfId !== undefined) this.gltfId = json.gltfId;
    if (json.mode !== undefined) this.mode = json.mode;
    if (json.indicesId !== undefined) this.indicesId = json.indicesId;
    if (json.weights !== undefined) this.weights = json.weights;
    if (json.morphPositionIds !== undefined)
      this.morphPositionIds = json.morphPositionIds;
    if (json.morphNormalIds !== undefined)
      this.morphNormalIds = json.morphNormalIds;
    if (json.morphTangentIds !== undefined)
      this.morphTangentIds = json.morphTangentIds;
    if (json.POSITION !== undefined) this.POSITION = json.POSITION;
    if (json.NORMAL !== undefined) this.NORMAL = json.NORMAL;
    if (json.TANGENT !== undefined) this.TANGENT = json.TANGENT;
    if (json.TEXCOORD_0 !== undefined) this.TEXCOORD_0 = json.TEXCOORD_0;
    if (json.TEXCOORD_1 !== undefined) this.TEXCOORD_1 = json.TEXCOORD_1;
    if (json.COLOR_0 !== undefined) this.COLOR_0 = json.COLOR_0;
    if (json.JOINTS_0 !== undefined) this.JOINTS_0 = json.JOINTS_0;
    if (json.WEIGHTS_0 !== undefined) this.WEIGHTS_0 = json.WEIGHTS_0;
    if (json.skin !== undefined) this.skin = json.skin;
  }

  static fromJSON(json: PrimitiveJSON) {
    const mesh = new Primitive(json.id);
    mesh.applyJSON(json);
    return mesh;
  }
}
