import { POSITION_ARRAY_ROUNDING, ROTATION_ARRAY_ROUNDING } from "../constants";
import { Engine } from "../Engine";

/**
 * Represents a player in the game.
 */
export class Player {
  readonly id: number;
  readonly engine: Engine;

  #name: string | null = null;
  #grounded = false;
  #avatar: string | null = null;

  readonly position: Int32Array;
  readonly rotation: Int16Array;

  constructor(id: number, engine: Engine) {
    this.id = id;
    this.engine = engine;

    const positionBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 3);
    this.position = new Int32Array(positionBuffer);

    const rotationBuffer = new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 4);
    this.rotation = new Int16Array(rotationBuffer);

    engine.render.send({
      subject: "add_player",
      data: { id, position: this.position, rotation: this.rotation },
    });
  }

  get name() {
    return this.#name;
  }

  set name(value: string | null) {
    if (this.#name === value) return;
    this.#name = value;

    this.engine.render.send({
      subject: "set_player_name",
      data: { playerId: this.id, name: value },
    });
  }

  get grounded() {
    return this.#grounded;
  }

  set grounded(value: boolean) {
    if (this.#grounded === value) return;
    this.#grounded = value;

    this.engine.render.send({
      subject: "set_player_grounded",
      data: { playerId: this.id, grounded: value },
    });
  }

  get avatar() {
    return this.#avatar;
  }

  set avatar(value: string | null) {
    if (this.#avatar === value) return;
    this.#avatar = value;

    this.engine.render.send({
      subject: "set_player_avatar",
      data: { playerId: this.id, uri: value },
    });
  }

  setPosition(x: number, y: number, z: number) {
    Atomics.store(this.position, 0, x * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.position, 1, y * POSITION_ARRAY_ROUNDING);
    Atomics.store(this.position, 2, z * POSITION_ARRAY_ROUNDING);
  }

  setRotation(x: number, y: number, z: number, w: number) {
    Atomics.store(this.rotation, 0, x * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.rotation, 1, y * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.rotation, 2, z * ROTATION_ARRAY_ROUNDING);
    Atomics.store(this.rotation, 3, w * ROTATION_ARRAY_ROUNDING);
  }
}
