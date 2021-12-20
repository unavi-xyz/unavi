import sdk from "matrix-js-sdk";

const withHttps = (url: string) =>
  !/^https?:\/\//i.test(url) ? `https://${url}` : url;

export async function login(
  homeserver: string,
  user: string,
  password: string
) {
  try {
    const client = sdk.createClient(withHttps(homeserver));
    const { user_id, device_id, access_token } = await client.login(
      "m.login.password",
      {
        user,
        password,
      }
    );
  } catch (e) {
    return e;
  }
}

export function signup(
  homeserver: string,
  username: string,
  password: string
) {}

export function logout() {}

export function guestLogin() {}
