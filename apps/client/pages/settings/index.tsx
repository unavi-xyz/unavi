import { useEffect, useRef, useState } from "react";
import { nanoid } from "nanoid";

import { useProfileByHandle } from "../../src/helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../src/helpers/lens/store";
import { AttributeData, ProfileMetadata } from "../../src/helpers/lens/types";
import { useEthersStore } from "../../src/helpers/ethers/store";
import { updateMetadata } from "../../src/helpers/lens/actions/updateMetadata";
import { updateProfilePicture } from "../../src/helpers/lens/actions/updateProfilePicture";
import { useMediaImage } from "../../src/helpers/lens/hooks/useMediaImage";
import { uploadFileToIpfs } from "../../src/helpers/ipfs/fetch";

import TextField from "../../src/components/base/TextField";
import TextArea from "../../src/components/base/TextArea";
import SettingsLayout from "../../src/components/layouts/SettingsLayout/SettingsLayout";
import Button from "../../src/components/base/Button";
import FileUpload from "../../src/components/base/FileUpload";

export default function Settings() {
  const nameRef = useRef<HTMLInputElement>(null);
  const locationRef = useRef<HTMLInputElement>(null);
  const websiteRef = useRef<HTMLInputElement>(null);
  const twitterRef = useRef<HTMLInputElement>(null);
  const bioRef = useRef<HTMLTextAreaElement>(null);

  const [pfpFile, setPfpFile] = useState<File>();
  const [pfpUrl, setPfpUrl] = useState<string>();
  const [coverFile, setCoverFile] = useState<File>();
  const [coverUrl, setCoverUrl] = useState<string>();
  const [loadingProfile, setLoadingProfile] = useState(false);
  const [loadingProfilePicture, setLoadingProfilePicture] = useState(false);

  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);
  const { url: pfpMediaUrl } = useMediaImage(profile?.picture);
  const { url: coverMediaUrl } = useMediaImage(profile?.coverPicture);

  useEffect(() => {
    setPfpUrl(pfpMediaUrl);
  }, [pfpMediaUrl]);

  useEffect(() => {
    setCoverUrl(coverMediaUrl);
  }, [coverMediaUrl]);

  useEffect(() => {
    if (pfpFile) setPfpUrl(URL.createObjectURL(pfpFile));
  }, [pfpFile]);

  useEffect(() => {
    if (coverFile) setCoverUrl(URL.createObjectURL(coverFile));
  }, [coverFile]);

  async function handleProfileSave() {
    const signer = useEthersStore.getState().signer;
    if (!signer || !profile) return;

    setLoadingProfile(true);

    try {
      //upload cover to ipfs
      const cover_picture: ProfileMetadata["cover_picture"] = coverFile
        ? await uploadFileToIpfs(coverFile)
        : profile.coverPicture?.__typename === "MediaSet"
        ? profile.coverPicture.original.url
        : profile.coverPicture?.__typename === "NftImage"
        ? profile.coverPicture.uri
        : null;

      //create metadata
      const attributes: ProfileMetadata["attributes"] =
        profile?.attributes?.map((attribute) => {
          const data: AttributeData = {
            key: attribute.key,
            value: attribute.value,
            traitType: attribute.traitType ?? undefined,
            displayType: (attribute.displayType as any) ?? undefined,
          };
          return data;
        }) ?? [];

      const metadata: ProfileMetadata = {
        version: "1.0.0",
        metadata_id: nanoid(),
        name: nameRef.current?.value ?? "",
        bio: bioRef.current?.value ?? "",
        location: locationRef.current?.value ?? "",
        cover_picture,
        social: [
          { key: "website", value: websiteRef.current?.value ?? "" },
          { key: "twitter", value: twitterRef.current?.value ?? "" },
        ],
        attributes,
      };

      //upload metadata
      await updateMetadata(profile, metadata);
    } catch (e) {
      console.error(e);
      setLoadingProfile(false);
    }

    setLoadingProfile(false);
  }

  async function handleProfilePictureSave() {
    if (!pfpFile || !profile) return;

    setLoadingProfilePicture(true);

    try {
      //update profile picture
      await updateProfilePicture(profile, pfpFile);
    } catch (e) {
      console.error(e);
      setLoadingProfilePicture(false);
    }

    setLoadingProfilePicture(false);
  }

  if (!profile) return null;

  return (
    <div className="space-y-8">
      <div className="space-y-8 bg-white rounded-2xl border p-8">
        <div className="text-lg space-y-4">
          <div className="flex items-center space-x-4 text-neutral-500">
            <div className="font-bold">Profile ID:</div>
            <div className="bg-neutral-100 px-2 py-1 rounded-xl border">
              {profile.id}
            </div>
          </div>

          <TextField
            inputRef={nameRef}
            title="Name"
            defaultValue={profile?.name}
          />
          <TextField
            inputRef={locationRef}
            title="Location"
            defaultValue={profile?.location}
          />
          <TextField
            inputRef={websiteRef}
            title="Website"
            defaultValue={profile?.website}
          />
          <TextField
            inputRef={twitterRef}
            title="Twitter"
            defaultValue={profile?.twitter}
          />
          <TextArea
            textAreaRef={bioRef}
            title="Bio"
            defaultValue={profile?.bio}
          />

          <div className="space-y-4">
            <div className="text-lg font-bold">Cover</div>

            {coverUrl && (
              <div className="w-full h-40">
                <img
                  src={coverUrl}
                  alt="cover picture preview"
                  className="object-cover rounded-xl h-full w-full border bg-neutral-100"
                />
              </div>
            )}

            <FileUpload
              title="Cover Picture"
              accept="image/*"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) setCoverFile(file);
              }}
            />
          </div>
        </div>

        <Button onClick={handleProfileSave} loading={loadingProfile}>
          Save
        </Button>
      </div>

      <div className="space-y-8 bg-white rounded-2xl border p-8">
        <div className="font-bold text-xl">Profile Picture</div>

        {pfpUrl && (
          <div className="flex space-x-8">
            <div className="w-40 h-40">
              <img
                src={pfpUrl}
                alt="profile picture preview square"
                className="object-cover rounded-xl h-full w-full border bg-neutral-100"
              />
            </div>
            <div className="w-40 h-40">
              <img
                src={pfpUrl}
                alt="profile picture preview circle"
                className="object-cover rounded-full h-full w-full border bg-neutral-100"
              />
            </div>
          </div>
        )}

        <div>
          <FileUpload
            title="Profile Picture"
            accept="image/*"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) setPfpFile(file);
            }}
          />
        </div>

        <Button
          onClick={handleProfilePictureSave}
          loading={loadingProfilePicture}
          disabled={!pfpFile}
        >
          Save
        </Button>
      </div>
    </div>
  );
}

Settings.Layout = SettingsLayout;
