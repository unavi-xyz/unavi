import { Player } from "3d";
import World from "./World";

export default function Scene() {
  return (
    <group>
      <Player />
      <World />
    </group>
  );
}
