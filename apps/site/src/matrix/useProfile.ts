import { IMatrixProfile } from "matrix-js-sdk";
import { useEffect, useState } from "react";

export default function useProfile(client, userId) {
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
