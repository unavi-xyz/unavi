import { struct } from "thyseus";

@struct
export class PlayerJoin {
  @struct.u8 declare playerId: number;
}

@struct
export class PlayerLeave {
  @struct.u8 declare playerId: number;
}
