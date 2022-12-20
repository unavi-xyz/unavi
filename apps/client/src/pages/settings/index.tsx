import { useConnectModal } from "@rainbow-me/rainbowkit";
import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import React, { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import { getNavbarLayout } from "../../home/layouts/NavbarLayout/NavbarLayout";
import Button from "../../ui/Button";
import Spinner from "../../ui/Spinner";
import TextField from "../../ui/TextField";

export default function Settings() {
  const [username, setUsername] = useState("");
  const [savingUsername, setSavingUsername] = useState(false);

  const { data: session, status } = useSession();
  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();

  const { data: profile, isLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  useEffect(() => {
    if (profile?.handle) setUsername(profile.handle.string);
  }, [profile]);

  const usernameDisabled =
    savingUsername || username.length === 0 || username === profile?.handle?.string;

  async function saveUsername() {
    if (usernameDisabled) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setSavingUsername(true);

    const contract = Profile__factory.connect(PROFILE_ADDRESS, signer);

    // If the user doesn't have a profile, create one
    if (!profile) {
      try {
        const tx = await contract.mintWithHandle(username);

        toast.promise(tx.wait(), {
          loading: "Creating profile...",
          success: "Profile created.",
          error: "Failed to create profile.",
        });

        await tx.wait();
      } catch (err) {
        console.error(err);
      }
    } else {
      // If the user has a profile, update the username
      try {
        const tx = await contract.setHandle(profile.id, username);

        toast.promise(tx.wait(), {
          loading: "Updating username...",
          success: "Username updated.",
          error: "Failed to update username.",
        });

        await tx.wait();
      } catch (err) {
        console.error(err);
      }
    }

    setSavingUsername(false);
  }

  if (status !== "authenticated") return null;

  if (isLoading)
    return (
      <div className="flex justify-center pt-12">
        <Spinner />
      </div>
    );

  return (
    <div className="max-w-content mx-auto pt-12">
      <div className="text-center text-3xl font-black">Edit Profile</div>

      <div className="mx-auto w-1/2">
        <div className="space-y-2">
          <TextField
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            name="Username"
            autoComplete="off"
            disabled={savingUsername}
            outline
          />

          <div className="flex justify-end">
            <Button variant="tonal" onClick={saveUsername} disabled={usernameDisabled}>
              Save
            </Button>
          </div>
        </div>

        <div className="space-y-2"></div>
      </div>
    </div>
  );
}

Settings.getLayout = getNavbarLayout;
