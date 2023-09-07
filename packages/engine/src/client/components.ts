import { Transform } from "lattice-engine/scene";
import { f32, struct, u8 } from "thyseus";

@struct
export class WorldJson {
  host: string = "";
}

@struct
export class OtherPlayer {
  id: u8 = 0;
}

@struct
export class NetworkTransform {
  transform: Transform = new Transform();
  lastUpdate: f32 = 0;
}

@struct
export class PrevTranslation {
  x: f32 = 0;
  y: f32 = 0;
  z: f32 = 0;
}
