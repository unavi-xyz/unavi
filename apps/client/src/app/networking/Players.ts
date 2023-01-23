import { Player } from "./Player";

export class Players {
  #store = new Map<string, Player>();

  constructor() {}
}
