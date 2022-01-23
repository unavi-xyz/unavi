import sdk from "matrix-js-sdk";
import { DEFAULT_HOMESERVER, initClient } from "..";

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
