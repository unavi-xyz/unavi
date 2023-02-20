import { Engine } from "../Engine";
import { Player } from "./Player";

/**
 * Manages the lifecycle of players.
 *
 * @group modules
 */
export class PlayerModules {
  readonly engine: Engine;

  #players = new Map<number, Player>();

  constructor(engine: Engine) {
    this.engine = engine;
  }

  addPlayer(id: number) {
    const player = new Player(id, this.engine);
    this.#players.set(id, player);
    return player;
  }

  getPlayer(id: number) {
    return this.#players.get(id);
  }

  removePlayer(id: number) {
    this.#players.delete(id);
    this.engine.render.send({ subject: "remove_player", data: id });
  }
}
