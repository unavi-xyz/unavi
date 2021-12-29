import { useEffect, useState } from "react";
import { IMatrixProfile, MatrixClient } from "matrix-js-sdk";

export function useProfile(client: MatrixClient, userId: string) {
  const [profile, setProfile] = useState<null | IMatrixProfile>(null);

  useEffect(() => {
    if (!client || !userId) return;

    client
      .getProfileInfo(String(userId))
      .then((res) => {
        setProfile(res);
      })
      .catch(() => {
        setProfile(null);
      });
  }, [client, userId]);

  return profile;
}
