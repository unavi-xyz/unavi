import { ChangeEvent, useEffect, useState } from "react";

import { Button, TextField } from "../../../base";
import { useCreateProfile } from "../../../../helpers/lens/hooks/useCreateProfile";
import { useValidateHandle } from "../../../../helpers/lens/hooks/useValidateHandle";
import { useLensStore } from "../../../../helpers/lens/store";

interface Props {
  handleClose: () => void;
}

export default function CreateProfilePage({ handleClose }: Props) {
  const [formHandle, setFormHandle] = useState("");
  const [handle, setHandle] = useState("");
  const [loadingSubmit, setLoadingSubmit] = useState(false);

  const { valid, loading } = useValidateHandle(handle);
  const { createProfile } = useCreateProfile();

  useEffect(() => {
    //give a delay before re-querying handle validation
    const timeout = setTimeout(() => {
      setHandle(formHandle);
    }, 750);

    return () => {
      clearTimeout(timeout);
    };
  }, [formHandle]);

  function handleUsernameChange(e: ChangeEvent<HTMLInputElement>) {
    const value = e.target.value;
    setFormHandle(value);
  }

  async function handleSubmit() {
    if (!valid) return;
    setLoadingSubmit(true);

    try {
      await createProfile(handle);
      useLensStore.setState({ handle });

      setLoadingSubmit(false);
      handleClose();
    } catch {
      setLoadingSubmit(false);
    }
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Create a Profile</h1>
        <p className="text-lg flex justify-center">Mint a new profile NFT.</p>
      </div>

      <TextField
        title="Username"
        frontAdornment="@"
        maxLength={31}
        onChange={handleUsernameChange}
      />

      <Button
        disabled={!valid || loading || formHandle !== handle || loadingSubmit}
        loading={loading || formHandle !== handle || loadingSubmit}
        onClick={handleSubmit}
      >
        Submit
      </Button>
    </div>
  );
}
