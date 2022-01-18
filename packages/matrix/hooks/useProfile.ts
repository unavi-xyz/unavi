import { MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

export function useProfile(client: MatrixClient, id: string) {
  async function fetcher() {
    const profile = await client.getProfileInfo(id);
    const picture = profile.avatar_url
      ? client.mxcUrlToHttp(profile.avatar_url)
      : null;

    return { profile, picture };
  }

  const { data } = useSWR(`profile-${id}`, fetcher);

  return data ?? { profile: null, picture: null };
}
