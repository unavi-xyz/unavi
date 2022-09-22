import { useRef, useState } from "react";

import { useLens } from "../../../lib/lens/hooks/useLens";
import { useProfileByHandle } from "../../../lib/lens/hooks/useProfileByHandle";
import { useSetProfileMetadata } from "../../../lib/lens/hooks/useSetProfileMetadata";
import { createProfileMetadata } from "../../../lib/lens/utils/createProfileMetadata";
import Button from "../../../ui/base/Button";
import TextField from "../../../ui/base/TextField";

export default function HostPage() {
  const hostRef = useRef<HTMLInputElement>(null);

  const { handle } = useLens();
  const profile = useProfileByHandle(handle);
  const setProfileMetadata = useSetProfileMetadata(profile?.id);

  const [loading, setLoading] = useState(false);

  async function handleSubmit() {
    if (loading) return;
    setLoading(true);

    const { metadata, updateAttribute } = createProfileMetadata(profile);
    updateAttribute("host", hostRef.current?.value ?? "host.thewired.space");

    try {
      await setProfileMetadata(metadata);
      // setDidSetHost(true);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="flex justify-center text-3xl">Host Server</h1>
        <p className="text-center">
          Specify a server to host your spaces. Can be changed at any time.
          (defaults to <b>host.thewired.space</b>)
        </p>
      </div>

      <TextField
        inputRef={hostRef}
        autoComplete="off"
        title="Host Server"
        defaultValue="host.thewired.space"
      />

      <div className="flex justify-end">
        <Button onClick={handleSubmit} variant="filled" loading={loading}>
          Submit
        </Button>
      </div>
    </div>
  );
}
