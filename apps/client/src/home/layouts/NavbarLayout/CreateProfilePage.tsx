import { useCreateProfileMutation } from "lens";
import { useEffect, useState } from "react";

import { HANDLE_ENDING } from "../../../client/lens/constants";
import { useLens } from "../../../client/lens/hooks/useLens";
import { useValidateHandle } from "../../../client/lens/hooks/useValidateHandle";
import Button from "../../../ui/Button";
import ErrorBox from "../../../ui/ErrorBox";
import TextField from "../../../ui/TextField";

interface Props {
  onClose: () => void;
}

export default function CreateProfilePage({ onClose }: Props) {
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

      // Close the modal
      onClose();
    } catch (err: any) {
      console.error(err);
      setError(err.message);
      setLoadingSubmit(false);
    }

    setLoadingSubmit(false);
  }

  return (
    <div className="space-y-6">
      <h1 className="text-center text-3xl font-bold">Create Your Profile</h1>

      <div className="space-y-4">
        <TextField
          title="Handle"
          frontAdornment="@"
          maxLength={31 - HANDLE_ENDING.length}
          value={formHandle}
          outline
          onChange={(e) => {
            const value = e.target.value;
            if (value.match("^[a-zA-Z0-9_.]*$")) {
              setFormHandle(value.toLowerCase());
            }
          }}
        />

        <ErrorBox error={error} />

        <div className="flex justify-end">
          <div className="w-full md:w-min">
            <Button
              variant="filled"
              fullWidth
              disabled={disabled}
              loading={loading}
              onClick={handleSubmit}
            >
              Submit
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
