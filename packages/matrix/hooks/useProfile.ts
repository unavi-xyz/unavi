import { useEffect, useState } from "react";
import { IMatrixProfile, MatrixClient } from "matrix-js-sdk";

export function useProfile(client: MatrixClient | null, userId: string | null) {
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
