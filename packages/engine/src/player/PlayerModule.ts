import { Player } from "./Player";

export class PlayerModule {
  store = new Map<string, Player>();

  constructor() {}

  addPlayer(player: Player) {}
}
