import sdk, { MatrixClient } from "matrix-js-sdk";
import { DEFAULT_HOMESERVER, EVENTS, initClient, STATE_KEY } from "..";

export async function getGuestClient() {
  const tmpClient = sdk.createClient(DEFAULT_HOMESERVER);
  const { user_id, device_id, access_token } = await tmpClient.registerGuest(
    {}
  );

  const client = await initClient(
    DEFAULT_HOMESERVER,
    access_token,
    device_id,
    user_id,
    true
  );

  return client;
}

export async function setState(
  client: MatrixClient,
  roomId: string,
  event: EVENTS,
  value: string
) {
  const res = await client.sendStateEvent(roomId, event, { value }, STATE_KEY);
  return res;
}

export async function getState(
  client: MatrixClient,
  roomId: string,
  event: EVENTS
) {
  const { value } = await client.getStateEvent(roomId, event, STATE_KEY);
  return value;
}
