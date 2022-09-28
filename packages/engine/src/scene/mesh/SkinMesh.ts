import { BehaviorSubject } from "rxjs";

import { SkinMeshJSON } from "./types";

export class SkinMesh {
  readonly type = "Skin";

  inverseBindMatricesId$ = new BehaviorSubject<string | null>(null);
  jointIds$ = new BehaviorSubject<string[]>([]);

  mode$ = new BehaviorSubject(4);
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

  get inverseBindMatricesId() {
    return this.inverseBindMatricesId$.value;
  }

  set inverseBindMatricesId(inverseBindMatricesId: string | null) {
    this.inverseBindMatricesId$.next(inverseBindMatricesId);
  }

  get jointIds() {
    return this.jointIds$.value;
  }

  set jointIds(jointIds: string[]) {
    this.jointIds$.next(jointIds);
  }

  get mode() {
    return this.mode$.value;
  }

  set mode(mode: number) {
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
    this.inverseBindMatricesId$.complete();
    this.jointIds$.complete();
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

  toJSON(): SkinMeshJSON {
    return {
      type: this.type,
      inverseBindMatricesId: this.inverseBindMatricesId,
      jointIds: this.jointIds,
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
    };
  }

  applyJSON(json: Partial<SkinMeshJSON>) {
    if (json.inverseBindMatricesId !== undefined)
      this.inverseBindMatricesId = json.inverseBindMatricesId;
    if (json.jointIds !== undefined) this.jointIds = json.jointIds;
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
  }

  static fromJSON(json: SkinMeshJSON) {
    const mesh = new SkinMesh();
    mesh.applyJSON(json);
    return mesh;
  }
}
