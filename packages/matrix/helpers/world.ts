import { MatrixClient } from "matrix-js-sdk";
import { EVENTS, STATE_KEY } from "..";

export async function setScene(
  client: MatrixClient,
  roomId: string,
  scene: string
) {
  const res = await client.sendStateEvent(
    roomId,
    EVENTS.scene,
    { scene },
    STATE_KEY
  );
  return res;
}

export async function getScene(client: MatrixClient, roomId: string) {
  const scene = await client.getStateEvent(roomId, EVENTS.scene, STATE_KEY);
  return scene;
}
