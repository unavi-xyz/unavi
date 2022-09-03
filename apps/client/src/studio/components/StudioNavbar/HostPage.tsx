import { useSetAtom } from "jotai";
import React, { useContext, useRef, useState } from "react";

import {
  LensContext,
  createProfileMetadata,
  useProfileByHandle,
  useSetProfileMetadata,
} from "@wired-xr/lens";

import Button from "../../../ui/base/Button";
import TextField from "../../../ui/base/TextField";
import { didSetHostAtom } from "./PublishPage";

export default function HostPage() {
  const hostRef = useRef<HTMLInputElement>(null);

  const { handle } = useContext(LensContext);
  const profile = useProfileByHandle(handle);
  const setProfileMetadata = useSetProfileMetadata(profile?.id);
  const setDidSetHost = useSetAtom(didSetHostAtom);

  const [loading, setLoading] = useState(false);

  async function handleSubmit() {
    if (loading) return;
    setLoading(true);

    const { metadata, updateAttribute } = createProfileMetadata(profile);
    updateAttribute("host", hostRef.current?.value ?? "host.thewired.space");

    try {
      await setProfileMetadata(metadata);
      setDidSetHost(true);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Host Server</h1>
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
