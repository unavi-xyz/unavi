import { struct } from "thyseus";

@struct
export class WorldJson {
  @struct.string declare host: string;
}
