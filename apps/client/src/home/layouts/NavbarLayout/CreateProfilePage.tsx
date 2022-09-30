import { useCreateProfileMutation } from "@wired-labs/lens";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

import { useLens } from "../../../lib/lens/hooks/useLens";
import { useValidateHandle } from "../../../lib/lens/hooks/useValidateHandle";
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
  const [, createProfile] = useCreateProfileMutation();

  const { switchProfile } = useLens();

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
    }, 500);

    return () => {
      clearTimeout(timeout);
    };
  }, [formHandle, setHandle]);

  async function handleSubmit() {
    if (disabled || loading) return;
    setLoadingSubmit(true);

    try {
      // Create profile
      const { data, error } = await createProfile({
        request: {
          handle,
          followModule: null,
          followNFTURI: null,
          profilePictureUri: null,
        },
      });

      if (data?.createProfile.__typename === "RelayError")
        throw new Error(data.createProfile.reason);
      if (error) throw new Error(error.message);

      // Log the user in
      switchProfile(handle);

      // Redirect to the new profile page
      router.push(`/user/${handle}`);
    } catch (err: any) {
      console.error(err);
      setError(err.message);
      setLoadingSubmit(false);
    }

    setLoadingSubmit(false);
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="flex justify-center text-3xl">Create a Profile</h1>
        <p className="flex justify-center text-lg">Mint a Lens profile NFT</p>
      </div>

      <div className="space-y-4">
        <TextField
          title="Handle"
          frontAdornment="@"
          maxLength={31}
          value={formHandle}
          outline
          onChange={(e) => {
            const value = e.target.value;
            if (value.match("^[a-zA-Z0-9_.]*$")) {
              setFormHandle(value.toLowerCase());
            }
          }}
        />

        <div className="flex w-full justify-end">
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
