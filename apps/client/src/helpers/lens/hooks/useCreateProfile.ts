import { useMutation } from "@apollo/client";
import {
  CreateProfileMutation,
  CreateProfileMutationVariables,
} from "../../../generated/graphql";

import { apolloClient } from "../apollo";
import { authenticate } from "../authentication";
import { CREATE_PROFILE } from "../queries";
import { useLensStore } from "../store";

export function useCreateProfile() {
  const [mutateFunction, { data, loading }] = useMutation<
    CreateProfileMutation,
    CreateProfileMutationVariables
  >(CREATE_PROFILE, { client: apolloClient });

  async function createProfile(handle: string) {
    if (!useLensStore.getState().authenticated) await authenticate();
    await mutateFunction({ variables: { handle } });
  }

  return { createProfile, data, loading };
}
