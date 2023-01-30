import { useConnectModal } from "@rainbow-me/rainbowkit";
import { ERC721Metadata, Profile__factory, PROFILE_ADDRESS } from "contracts";
import React, { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { useSession } from "../client/auth/useSession";
import { trpc } from "../client/trpc";
import { cropImage } from "../editor/utils/cropImage";
import { env } from "../env/client.mjs";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../home/MetaTags";
import ProfilePicture from "../home/ProfilePicture";
import Button from "../ui/Button";
import ButtonFileInput from "../ui/ButtonFileInput";
import Spinner from "../ui/Spinner";
import TextArea from "../ui/TextArea";
import TextField from "../ui/TextField";
import { numberToHexDisplay } from "../utils/numberToHexDisplay";

function getProfileMetadataURL(profileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/profiles/${profileId}/metadata.json`;
}

function getProfileImageURL(profileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/profiles/${profileId}/image.jpg`;
}

function getProfileCoverImageURL(profileId: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/profiles/${profileId}/cover.jpg`;
}

export default function Settings() {
  const [name, setName] = useState("");
  const [savingName, setSavingName] = useState(false);

  const [bio, setBio] = useState("");
  const [profilePicture, setProfilePicture] = useState("");
  const [coverImage, setCoverImage] = useState("");
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
    trpc.social.profile.getMetadataUpload.useMutation();
  const { mutateAsync: createImageUploadURL } = trpc.social.profile.getImageUpload.useMutation();
  const { mutateAsync: createCoverImageUploadURL } =
    trpc.social.profile.getCoverUpload.useMutation();

  useEffect(() => {
    if (profile?.handle) setName(profile.handle.string);
    if (profile?.metadata) {
      setBio(profile.metadata.description);
      setProfilePicture(profile.metadata.image);
      setCoverImage(profile.metadata.animation_url);
    }
  }, [profile]);

  const nameDisabled = savingName || name.length === 0 || name === profile?.handle?.string;
  const metadataDisabled = savingMetadata;

  async function handleSaveName() {
    if (nameDisabled) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setSavingName(true);

    const contract = Profile__factory.connect(PROFILE_ADDRESS, signer);

    // If the user doesn't have a profile, create one
    if (!profile) {
      try {
        const tx = await contract.mintWithHandle(name);

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
      // If the user has a profile, update the name
      try {
        const tx = await contract.setHandle(profile.id, name);

        toast.promise(tx.wait(), {
          loading: "Updating name...",
          success: "Name updated.",
          error: "Failed to update name.",
        });

        await tx.wait();
      } catch (err) {
        console.error(err);
      }
    }

    setSavingName(false);
  }

  async function handleSaveMetadata() {
    if (metadataDisabled) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setSavingMetadata(true);

    async function saveMetadata() {
      if (!profile) throw new Error("No profile found");
      if (!signer) throw new Error("No signer found");

      const hexId = numberToHexDisplay(profile.id);

      const metadata: ERC721Metadata = {
        animation_url: coverImage,
        description: bio,
        external_url: `https://thewired.space/user/${hexId}`,
        image: profilePicture,
        name,
      };

      // Upload profile picture to S3 if it's a new one
      const isBlob = profilePicture.startsWith("blob:");
      if (isBlob) {
        const imageUrl = await createImageUploadURL({ id: profile.id });

        const body = await fetch(profilePicture).then((res) => res.blob());

        const imageResponse = await fetch(imageUrl, {
          method: "PUT",
          body,
          headers: {
            "Content-Type": "image/png",
            "x-amz-acl": "public-read",
          },
        });

        if (!imageResponse.ok) throw new Error("Failed to upload profile picture");

        metadata.image = getProfileImageURL(hexId);
      }

      // Upload cover image to S3 if it's a new one
      const isCoverBlob = coverImage.startsWith("blob:");
      if (isCoverBlob) {
        const coverImageUrl = await createCoverImageUploadURL({ id: profile.id });

        const coverBody = await fetch(coverImage).then((res) => res.blob());

        const coverImageResponse = await fetch(coverImageUrl, {
          method: "PUT",
          body: coverBody,
          headers: {
            "Content-Type": "image/png",
            "x-amz-acl": "public-read",
          },
        });

        if (!coverImageResponse.ok) throw new Error("Failed to upload cover image");

        metadata.animation_url = getProfileCoverImageURL(hexId);
      }

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
      const contract = Profile__factory.connect(PROFILE_ADDRESS, signer);
      const currentTokenURI = await contract.tokenURI(profile.id);

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
              value={name}
              onChange={(e) => setName(e.target.value)}
              name="Name"
              autoComplete="off"
              disabled={savingName}
              outline
            />

            <div className="flex justify-end">
              <Button variant="tonal" onClick={handleSaveName} disabled={nameDisabled}>
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

              <div className="space-y-2">
                <div className="text-lg font-bold">Profile Picture</div>

                <div className="flex justify-center pb-2">
                  <div className="flex w-1/2 justify-center">
                    <ProfilePicture
                      size={256}
                      circle
                      src={profilePicture}
                      uniqueKey={profile?.handle?.full ?? session?.address ?? ""}
                    />
                  </div>
                </div>

                <ButtonFileInput
                  disabled={savingMetadata}
                  onChange={async (e) => {
                    if (!e.target.files) return;

                    const file = e.target.files[0];
                    if (!file) return;

                    const url = URL.createObjectURL(file);

                    // Crop image
                    const croppedFile = await cropImage(url, 1);
                    const croppedUrl = URL.createObjectURL(croppedFile);

                    setProfilePicture(croppedUrl);
                  }}
                >
                  Upload Image
                </ButtonFileInput>

                {profilePicture && (
                  <Button
                    fullWidth
                    rounded="small"
                    color="error"
                    disabled={savingMetadata}
                    onClick={() => {
                      if (savingMetadata) return;
                      setProfilePicture("");
                    }}
                  >
                    Remove Image
                  </Button>
                )}
              </div>

              <div className="space-y-2">
                <div className="text-lg font-bold">Cover Picture</div>

                <div className="flex justify-center pb-2">
                  <div className="flex h-40 w-full justify-center rounded-xl bg-sky-100">
                    {coverImage && (
                      <img
                        src={coverImage}
                        alt=""
                        crossOrigin="anonymous"
                        className="h-40 w-full rounded-xl object-cover"
                      />
                    )}
                  </div>
                </div>

                <ButtonFileInput
                  disabled={savingMetadata}
                  onChange={async (e) => {
                    if (!e.target.files) return;

                    const file = e.target.files[0];
                    if (!file) return;

                    const url = URL.createObjectURL(file);

                    // Crop image
                    const croppedFile = await cropImage(url, 4);
                    const croppedUrl = URL.createObjectURL(croppedFile);

                    setCoverImage(croppedUrl);
                  }}
                >
                  Upload Image
                </ButtonFileInput>

                {coverImage && (
                  <Button
                    fullWidth
                    rounded="small"
                    color="error"
                    disabled={savingMetadata}
                    onClick={() => {
                      if (savingMetadata) return;
                      setCoverImage("");
                    }}
                  >
                    Remove Image
                  </Button>
                )}
              </div>

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
