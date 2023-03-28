"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { ERC721Metadata, Profile__factory, PROFILE_ADDRESS } from "contracts";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { cropImage } from "../../../src/editor/utils/cropImage";
import { env } from "../../../src/env.mjs";
import { Profile } from "../../../src/server/helpers/fetchProfile";
import Button from "../../../src/ui/Button";
import ImageInput from "../../../src/ui/ImageInput";
import TextArea from "../../../src/ui/TextArea";
import { cdnURL, S3Path } from "../../../src/utils/s3Paths";
import { toHex } from "../../../src/utils/toHex";
import { getProfileFileUpload } from "../../api/profiles/[id]/[file]/helper";

interface Props {
  profile: Profile | null;
}

export default function Metadata({ profile }: Props) {
  const [bio, setBio] = useState(profile?.metadata?.description ?? "");
  const [profilePicture, setProfilePicture] = useState(profile?.metadata?.image ?? "");
  const [coverImage, setCoverImage] = useState(profile?.metadata?.animation_url ?? "");
  const [saving, setSaving] = useState(false);

  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (saving) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    async function saveMetadata() {
      if (!profile) throw new Error("No profile found");
      if (!signer) throw new Error("No signer found");

      const hexId = toHex(profile.id);

      const metadata: ERC721Metadata = {
        animation_url: coverImage,
        description: bio,
        external_url: `${env.NEXT_PUBLIC_DEPLOYED_URL}/user/${hexId}`,
        image: profilePicture,
        name: profile.handle?.string ?? "",
      };

      // Upload profile picture to S3 if it's a new one
      if (profilePicture.startsWith("blob:")) {
        const imageUrl = await getProfileFileUpload(profile.id, "image");

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

        metadata.image = cdnURL(S3Path.profile(profile.id).image);
      }

      // Upload cover image to S3 if it's a new one
      const isCoverBlob = coverImage.startsWith("blob:");
      if (isCoverBlob) {
        const coverImageUrl = await getProfileFileUpload(profile.id, "cover");

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

        metadata.animation_url = cdnURL(S3Path.profile(profile.id).cover);
      }

      // Upload metadata to S3
      const url = await getProfileFileUpload(profile.id, "metadata");

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

      const targetTokenURI = cdnURL(S3Path.profile(profile.id).metadata);

      if (currentTokenURI !== targetTokenURI) {
        const tx = await contract.setTokenURI(profile.id, targetTokenURI);
        await tx.wait();
      }
    }

    setSaving(true);

    toast
      .promise(saveMetadata(), {
        loading: "Updating profile...",
        success: "Profile updated.",
        error: "Failed to update profile.",
      })
      .finally(() => setSaving(false));
  }

  return (
    <div className="mx-auto max-w-xl">
      {profile ? (
        <form onSubmit={handleSubmit} className="space-y-2">
          <TextArea
            value={bio}
            onChange={(e) => setBio(e.target.value)}
            name="Bio"
            autoComplete="off"
            rows={4}
            disabled={saving}
          />

          <ImageInput
            name="Profile Picture"
            src={profilePicture}
            disabled={saving}
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
            className="h-48 w-48 rounded-full object-cover"
          />

          <ImageInput
            name="Cover Picture"
            src={coverImage}
            disabled={saving}
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
            className="h-40 w-full rounded-xl object-cover"
          />

          <div className="flex justify-end">
            <Button disabled={saving} type="submit">
              Save
            </Button>
          </div>
        </form>
      ) : null}
    </div>
  );
}
