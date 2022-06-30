import { Router } from "mediasoup/node/lib/Router";
import { Socket } from "socket.io";

import { Player } from "./Player";
import { Space } from "./Space";

export class GameManager {
  public readonly router: Router;

  private _spaces = new Map<string, Space>();

  constructor(router: Router) {
    this.router = router;
  }

  public createPlayer(socket: Socket) {
    return new Player(socket, this);
  }

  public getSpace(spaceId: string) {
    return this._spaces.get(spaceId);
  }

  public getOrCreateSpace(spaceId: string) {
    const space = this.getSpace(spaceId);
    if (space) return space;

    //create space if it doesn't exist
    const newSpace = new Space(spaceId, this);
    this._spaces.set(spaceId, newSpace);
    return newSpace;
  }

  public deleteSpace(spaceId: string) {
    this._spaces.delete(spaceId);
  }
}
