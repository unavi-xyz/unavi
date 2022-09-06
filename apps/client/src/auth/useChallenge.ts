import { useEffect, useMemo, useState } from "react";

import {
  GetChallengeDocument,
  GetChallengeQuery,
  GetChallengeQueryVariables,
} from "@wired-labs/lens";

import { lensClient } from "../lib/lens/client";

export function useChallenge(address: string | undefined) {
  const challengePromise = useMemo(
    async () => await generateChallenge(address),
    [address]
  );

  const [challenge, setChallenge] = useState<string>();

  useEffect(() => {
    challengePromise.then(setChallenge);
  }, [challengePromise]);

  return challenge;
}

async function generateChallenge(address: string | undefined) {
  if (!address) return;

  const { data, error } = await lensClient
    .query<GetChallengeQuery, GetChallengeQueryVariables>(
      GetChallengeDocument,
      {
        request: { address },
      }
    )
    .toPromise();

  if (error) throw new Error(error.message);
  if (data === undefined) throw new Error("No challenge recieved");

  return data.challenge.text;
}
