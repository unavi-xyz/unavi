import { struct, u8 } from "thyseus";

@struct
export class PlayerJoin {
  playerId: u8 = 0;
}

@struct
export class PlayerLeave {
  playerId: u8 = 0;
}
