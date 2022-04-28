import { useEffect, useState } from "react";

import { useLensStore } from "../../../helpers/lens/store";
import { apolloClient } from "../../../helpers/lens/apollo";
import { authenticate } from "../../../helpers/lens/authentication";
import { useValidateHandle } from "../../../helpers/lens/hooks/useValidateHandle";
import { CREATE_PROFILE } from "../../../helpers/lens/queries";

import {
  CreateProfileMutation,
  CreateProfileMutationVariables,
} from "../../../generated/graphql";

import TextField from "../../base/TextField";
import Button from "../../base/Button";

export default function CreateProfilePage() {
  const [handle, setHandle] = useState<string>();
  const [formHandle, setFormHandle] = useState<string>();
  const [loadingSubmit, setLoadingSubmit] = useState(false);

  const { valid, loading } = useValidateHandle(handle);

  useEffect(() => {
    //debounce handle input
    const timeout = setTimeout(() => {
      setHandle(formHandle);
    }, 750);

    return () => {
      clearTimeout(timeout);
    };
  }, [formHandle]);

  async function handleSubmit() {
    if (!valid || loadingSubmit) return;
    setLoadingSubmit(true);

    //create the profile
    try {
      await authenticate();

      await apolloClient.mutate<
        CreateProfileMutation,
        CreateProfileMutationVariables
      >({
        mutation: CREATE_PROFILE,
        variables: {
          handle,
        },
      });

      //log the user in
      useLensStore.setState({ handle });
    } catch {}

    setLoadingSubmit(false);
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Create a Profile</h1>
        <p className="text-lg flex justify-center">Mint a new profile NFT</p>
      </div>

      <div className="space-y-4">
        <TextField
          title="Username"
          frontAdornment="@"
          maxLength={31}
          onChange={(e) => setFormHandle(e.target.value)}
        />

        <Button
          disabled={!valid || loading || formHandle !== handle || loadingSubmit}
          loading={loading || formHandle !== handle || loadingSubmit}
          onClick={handleSubmit}
        >
          Submit
        </Button>

        <div className="flex justify-center text-red-500 font-bold">
          {!valid && !loading && formHandle === handle && (
            <p>Error: Invalid handle</p>
          )}
        </div>
      </div>
    </div>
  );
}
