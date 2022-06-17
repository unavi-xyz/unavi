import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import {
  CreateProfileDocument,
  CreateProfileMutation,
  CreateProfileMutationVariables,
} from "../../../generated/graphql";
import { authenticate } from "../../../helpers/lens/authentication";
import { lensClient } from "../../../helpers/lens/client";
import { useValidateHandle } from "../../../helpers/lens/hooks/useValidateHandle";
import { useLensStore } from "../../../helpers/lens/store";
import Button from "../../base/Button";
import ErrorBox from "../../base/ErrorBox";
import TextField from "../../base/TextField";

export default function CreateProfilePage() {
  const router = useRouter();

  const [handle, setHandle] = useState<string>("");
  const [formHandle, setFormHandle] = useState<string>("");
  const [loadingSubmit, setLoadingSubmit] = useState(false);
  const [error, setError] = useState<string>();

  const { valid, error: validateError, fetching } = useValidateHandle(handle);

  const loading = fetching || formHandle !== handle || loadingSubmit;
  const disabled = !valid;

  useEffect(() => {
    if (validateError) setError(validateError);
  }, [validateError]);

  useEffect(() => {
    //debounce handle input
    const timeout = setTimeout(() => {
      setHandle(formHandle);

      if (formHandle.length < 5 && formHandle.length > 0) {
        setError("Handle must be at least 5 characters");
        return;
      }

      setError(undefined);
    }, 750);

    return () => clearTimeout(timeout);
  }, [formHandle]);

  async function handleSubmit() {
    if (disabled || loading) return;
    setLoadingSubmit(true);

    try {
      await authenticate();

      //create the profile
      const { error } = await lensClient
        .mutation<CreateProfileMutation, CreateProfileMutationVariables>(
          CreateProfileDocument,
          { handle }
        )
        .toPromise();

      if (error) throw new Error(error.message);

      //log the user in
      useLensStore.setState({ handle });

      //redirect to the profile page
      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
      setError(err as any);
      setLoadingSubmit(false);
    }

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
          title="Handle"
          frontAdornment="@"
          maxLength={31}
          value={formHandle}
          onChange={(e) => {
            if (e.target.value.match("^[a-zA-Z0-9_.]*$")) {
              setFormHandle(e.target.value.toLowerCase());
              return;
            }
          }}
        />

        <div className="flex justify-end w-full">
          <Button
            variant="filled"
            disabled={disabled}
            loading={loading}
            onClick={handleSubmit}
          >
            Submit
          </Button>
        </div>

        <ErrorBox error={error} />
      </div>
    </div>
  );
}
