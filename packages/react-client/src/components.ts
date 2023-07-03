import { Transform } from "lattice-engine/scene";
import { struct } from "thyseus";

@struct
export class WorldJson {
  @struct.string declare host: string;
}

@struct
export class OtherPlayer {
  @struct.u8 declare id: number;
}

@struct
export class NetworkTransform {
  @struct.substruct(Transform) declare transform: Transform;

  @struct.f32 declare lastUpdate: number;
}
