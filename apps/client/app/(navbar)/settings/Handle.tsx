"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { Profile } from "@/src/server/helpers/fetchProfile";
import Button from "@/src/ui/Button";
import TextField from "@/src/ui/TextField";

interface Props {
  profile: Profile | null;
}

export default function Handle({ profile }: Props) {
  const [username, setName] = useState(profile?.handle?.string ?? "");
  const [savingUsername, setSavingName] = useState(false);

  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();

  const disabled = savingUsername || username.length === 0 || username === profile?.handle?.string;

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (disabled) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setSavingName(true);

    const contract = Profile__factory.connect(PROFILE_ADDRESS, signer);

    // If the user doesn't have a profile, create one
    if (!profile) {
      try {
        const tx = await contract.mintWithHandle(username);

        await toast.promise(tx.wait(), {
          loading: "Creating profile...",
          success: "Profile created.",
          error: "Failed to create profile.",
        });
      } catch (err) {
        console.error(err);
      }
    } else {
      // If the user has a profile, update the name
      try {
        const tx = await contract.setHandle(profile.id, username);

        await toast.promise(tx.wait(), {
          loading: "Updating name...",
          success: "Name updated.",
          error: "Failed to update name.",
        });
      } catch (err) {
        console.error(err);
      }
    }

    setSavingName(false);
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-2">
      <TextField
        label="Username"
        name="username"
        autoComplete="off"
        value={username}
        disabled={savingUsername}
        onChange={(e) => setName(e.target.value)}
      />

      <div className="flex justify-end">
        <Button disabled={disabled} type="submit">
          Save
        </Button>
      </div>
    </form>
  );
}
