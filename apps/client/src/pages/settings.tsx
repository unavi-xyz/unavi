import { useConnectModal } from "@rainbow-me/rainbowkit";
import { ERC721Metadata, Profile__factory, PROFILE_ADDRESS } from "contracts";
import React, { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { useSession } from "../client/auth/useSession";
import { trpc } from "../client/trpc";
import { env } from "../env/client.mjs";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../home/MetaTags";
import Button from "../ui/Button";
import Spinner from "../ui/Spinner";
import TextArea from "../ui/TextArea";
import TextField from "../ui/TextField";
import { numberToHexDisplay } from "../utils/numberToHexDisplay";

function getProfileMetadataURL(profileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/profiles/${profileId}/metadata.json`;
}

export default function Settings() {
  const [username, setUsername] = useState("");
  const [savingUsername, setSavingUsername] = useState(false);

  const [bio, setBio] = useState("");
  const [savingMetadata, setSavingMetadata] = useState(false);

  const { data: session, status } = useSession();
  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();
  const utils = trpc.useContext();

  const { data: profile, isLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  const { mutateAsync: createMetadataUploadURL } =
    trpc.social.profile.metadataUploadURL.useMutation();

  useEffect(() => {
    if (profile?.handle) setUsername(profile.handle.string);
    if (profile?.metadata) {
      setBio(profile.metadata.description);
    }
  }, [profile]);

  const usernameDisabled =
    savingUsername || username.length === 0 || username === profile?.handle?.string;

  const metadataDisabled = savingMetadata || bio === profile?.metadata?.description;

  async function handleSaveUsername() {
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

  async function handleSaveMetadata() {
    if (metadataDisabled) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setSavingMetadata(true);

    const contract = Profile__factory.connect(PROFILE_ADDRESS, signer);

    async function saveMetadata() {
      if (!profile) throw new Error("No profile found");

      const metadata: ERC721Metadata = {
        animation_url: "",
        description: bio,
        external_url: "",
        image: "",
        name: username,
      };

      // Upload metadata to S3
      const url = await createMetadataUploadURL({ id: profile.id });

      const res = await fetch(url, {
        method: "PUT",
        body: JSON.stringify(metadata),
        headers: {
          "Content-Type": "application/json",
          "x-amz-acl": "public-read",
        },
      });

      if (!res.ok) throw new Error("Failed to upload metadata");

      // Update the tokenURI if not already set
      const currentTokenURI = await contract.tokenURI(profile.id);

      const hexId = numberToHexDisplay(profile.id);
      const targetTokenURI = getProfileMetadataURL(hexId);

      if (currentTokenURI !== targetTokenURI) {
        const tx = await contract.setTokenURI(profile.id, targetTokenURI);
        await tx.wait();
      }

      // Invalidate trpc query
      await Promise.all([
        utils.social.profile.byId.invalidate({ id: profile.id }),
        utils.social.profile.byAddress.invalidate({ address: session?.address ?? "" }),
      ]);

      if (profile.handle)
        await utils.social.profile.byHandle.invalidate({
          string: profile.handle.string,
          id: profile.id,
        });
    }

    toast.promise(
      saveMetadata().finally(() => {
        setSavingMetadata(false);
      }),
      {
        loading: "Updating profile...",
        success: "Profile updated.",
        error: "Failed to update profile.",
      }
    );
  }

  const hasProfile = !isLoading && profile !== undefined;

  if (status !== "authenticated") return null;

  if (isLoading)
    return (
      <div className="flex justify-center pt-12">
        <Spinner />
      </div>
    );

  return (
    <>
      <MetaTags title="Settings" />

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
              <Button variant="tonal" onClick={handleSaveUsername} disabled={usernameDisabled}>
                Save
              </Button>
            </div>
          </div>

          {hasProfile ? (
            <div className="space-y-2">
              <TextArea
                value={bio}
                onChange={(e) => setBio(e.target.value)}
                name="Bio"
                autoComplete="off"
                rows={4}
                disabled={savingMetadata}
                outline
              />

              <div className="flex justify-end">
                <Button variant="tonal" onClick={handleSaveMetadata} disabled={metadataDisabled}>
                  Save
                </Button>
              </div>
            </div>
          ) : null}
        </div>
      </div>
    </>
  );
}

Settings.getLayout = getNavbarLayout;
