"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { ProfileMetadata } from "@wired-protocol/types";
import { ERC721Metadata, Profile__factory, PROFILE_ADDRESS } from "contracts";
import { useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { getProfileFileUpload } from "@/app/api/profiles/[id]/[file]/helper";
import { cropImage } from "@/src/editor/utils/cropImage";
import { env } from "@/src/env.mjs";
import { Profile } from "@/src/server/helpers/fetchProfile";
import Button from "@/src/ui/Button";
import ImageInput from "@/src/ui/ImageInput";
import TextArea from "@/src/ui/TextArea";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";
import { toHex } from "@/src/utils/toHex";

interface Props {
  profile: Profile | null;
}

export default function Metadata({ profile }: Props) {
  const [bio, setBio] = useState<string | undefined>(profile?.metadata?.bio);
  const [image, setImage] = useState<string | undefined>(profile?.metadata?.image);
  const [background, setBackground] = useState<string | undefined>(profile?.metadata?.background);
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

    async function uploadImage() {
      if (!profile) throw new Error("No profile found");

      // Upload image to S3 if it's a new one
      if (image?.startsWith("blob:")) {
        const uploadURL = await getProfileFileUpload(profile.id, "image");

        const body = await fetch(image).then((res) => res.blob());

        const imageResponse = await fetch(uploadURL, {
          method: "PUT",
          body,
          headers: {
            "Content-Type": "image/png",
            "x-amz-acl": "public-read",
          },
        });

        if (!imageResponse.ok) throw new Error("Failed to upload image");

        return cdnURL(S3Path.profile(profile.id).image);
      } else {
        return image;
      }
    }

    async function uploadBackground() {
      if (!profile) throw new Error("No profile found");

      // Upload background to S3 if it's a new one
      if (background?.startsWith("blob:")) {
        const uploadURL = await getProfileFileUpload(profile.id, "background");

        const body = await fetch(background).then((res) => res.blob());

        const imageResponse = await fetch(uploadURL, {
          method: "PUT",
          body,
          headers: {
            "Content-Type": "image/png",
            "x-amz-acl": "public-read",
          },
        });

        if (!imageResponse.ok) throw new Error("Failed to upload background");

        return cdnURL(S3Path.profile(profile.id).background);
      } else {
        return background;
      }
    }

    const [imageURL, backgroundURL] = await Promise.all([uploadImage(), uploadBackground()]);

    async function saveMetadata() {
      if (!profile) throw new Error("No profile found");
      if (!signer) throw new Error("No signer found");

      const hexId = toHex(profile.id);

      // We only need ProfileMetadata for The Wired
      // But also support ERC721Metadata for OpenSea
      const metadata: ERC721Metadata & ProfileMetadata = {
        name: profile.handle?.string,
        bio,
        description: bio,
        image: imageURL,
        background: backgroundURL,
        animation_url: backgroundURL,
        external_url: `${env.NEXT_PUBLIC_DEPLOYED_URL}/user/${hexId}`,
      };

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
            label="Bio"
            name="bio"
            autoComplete="off"
            rows={4}
            disabled={saving}
            value={bio}
            onChange={(e) => setBio(e.target.value)}
          />

          <ImageInput
            name="Profile Picture"
            src={image}
            disabled={saving}
            onChange={async (e) => {
              if (!e.target.files) return;

              const file = e.target.files[0];
              if (!file) return;

              const url = URL.createObjectURL(file);

              // Crop image
              const croppedFile = await cropImage(url, 1);
              const croppedUrl = URL.createObjectURL(croppedFile);

              setImage(croppedUrl);
            }}
            className="h-48 w-48 rounded-full object-cover"
          />

          <ImageInput
            name="Background"
            src={background}
            disabled={saving}
            onChange={async (e) => {
              if (!e.target.files) return;

              const file = e.target.files[0];
              if (!file) return;

              const url = URL.createObjectURL(file);

              // Crop image
              const croppedFile = await cropImage(url, 4);
              const croppedUrl = URL.createObjectURL(croppedFile);

              setBackground(croppedUrl);
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
