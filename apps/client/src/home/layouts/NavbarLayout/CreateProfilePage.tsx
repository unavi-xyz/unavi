import { useRouter } from "next/router";
import { useContext, useEffect, useState } from "react";

import { LensContext, useValidateHandle } from "@wired-xr/lens";
import {
  CreateProfileDocument,
  CreateProfileMutation,
  CreateProfileMutationVariables,
} from "@wired-xr/lens";

import Button from "../../../ui/base/Button";
import ErrorBox from "../../../ui/base/ErrorBox";
import TextField from "../../../ui/base/TextField";

export default function CreateProfilePage() {
  const router = useRouter();

  const [handle, setHandle] = useState<string>("");
  const [formHandle, setFormHandle] = useState<string>("");
  const [loadingSubmit, setLoadingSubmit] = useState(false);
  const [error, setError] = useState<string>();

  const { valid, error: validateError, fetching } = useValidateHandle(handle);

  const {
    setHandle: switchProfile,
    client,
    authenticate,
  } = useContext(LensContext);

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

    return () => {
      clearTimeout(timeout);
    };
  }, [formHandle, setHandle]);

  async function handleSubmit() {
    if (disabled || loading) return;
    setLoadingSubmit(true);

    try {
      await authenticate();

      //create the profile
      const { error } = await client
        .mutation<CreateProfileMutation, CreateProfileMutationVariables>(
          CreateProfileDocument,
          {
            request: {
              handle,
              followModule: null,
              followNFTURI: null,
              profilePictureUri: null,
            },
          }
        )
        .toPromise();

      if (error) throw new Error(error.message);

      //log the user in
      switchProfile(handle);

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
        <p className="text-lg flex justify-center">Mint a Lens profile NFT</p>
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
