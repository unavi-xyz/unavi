import { nanoid } from "nanoid";
import { useEffect, useRef, useState } from "react";

import Button from "../../src/components/base/Button";
import FileUpload from "../../src/components/base/FileUpload";
import TextArea from "../../src/components/base/TextArea";
import TextField from "../../src/components/base/TextField";
import { getSettingsLayout } from "../../src/components/layouts/SettingsLayout/SettingsLayout";
import { uploadFileToIpfs } from "../../src/helpers/ipfs/fetch";
import { useMediaImage } from "../../src/helpers/lens/hooks/useMediaImage";
import { useProfileByHandle } from "../../src/helpers/lens/hooks/useProfileByHandle";
import { useSetProfileMetadata } from "../../src/helpers/lens/hooks/useSetProfileMetadata";
import { useSetProfilePicture } from "../../src/helpers/lens/hooks/useSetProfilePicture";
import { useLensStore } from "../../src/helpers/lens/store";
import {
  AttributeData,
  MetadataVersions,
  ProfileMetadata,
} from "../../src/helpers/lens/types";
import { crop } from "../../src/helpers/utils/crop";

export default function Settings() {
  const nameRef = useRef<HTMLInputElement>(null);
  const locationRef = useRef<HTMLInputElement>(null);
  const websiteRef = useRef<HTMLInputElement>(null);
  const twitterRef = useRef<HTMLInputElement>(null);
  const bioRef = useRef<HTMLTextAreaElement>(null);

  const [pfpRawFile, setPfpRawFile] = useState<File>();
  const [pfpFile, setPfpFile] = useState<File>();
  const [pfpUrl, setPfpUrl] = useState<string>();
  const [coverFile, setCoverFile] = useState<File>();
  const [coverUrl, setCoverUrl] = useState<string>();
  const [loadingProfile, setLoadingProfile] = useState(false);
  const [loadingProfilePicture, setLoadingProfilePicture] = useState(false);

  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);
  const pfpMediaUrl = useMediaImage(profile?.picture);
  const coverMediaUrl = useMediaImage(profile?.coverPicture);

  const setProfileMetadata = useSetProfileMetadata(profile?.id);
  const setProfilePicture = useSetProfilePicture(profile?.id);

  useEffect(() => {
    setPfpUrl(pfpMediaUrl);
  }, [pfpMediaUrl]);

  useEffect(() => {
    setCoverUrl(coverMediaUrl);
  }, [coverMediaUrl]);

  useEffect(() => {
    if (!pfpRawFile) return;

    crop(URL.createObjectURL(pfpRawFile), 1).then((res) => {
      setPfpFile(res);
    });
  }, [pfpRawFile]);

  useEffect(() => {
    if (!pfpFile) return;
    setPfpUrl(URL.createObjectURL(pfpFile));
  }, [pfpFile]);

  useEffect(() => {
    if (coverFile) setCoverUrl(URL.createObjectURL(coverFile));
  }, [coverFile]);

  async function handleProfileSave() {
    if (!profile || loadingProfile) return;

    setLoadingProfile(true);

    const cover_picture: string = coverFile
      ? await uploadFileToIpfs(coverFile)
      : profile.coverPicture?.__typename === "MediaSet"
      ? profile.coverPicture.original.url
      : profile.coverPicture?.__typename === "NftImage"
      ? profile.coverPicture.uri
      : null;

    const attributes =
      profile.attributes?.map((attribute) => {
        const data: AttributeData = {
          key: attribute.key,
          value: attribute.value,
          traitType: attribute.traitType ?? undefined,
          displayType: (attribute.displayType as any) ?? undefined,
        };
        return data;
      }) ?? [];

    function addAttribute(key: string, value: any) {
      const newData = {
        key,
        value,
      };

      const currentIndex = attributes.findIndex(
        (attribute) => attribute.key === key
      );

      if (value === undefined) {
        if (currentIndex !== -1) {
          //remove attribute
          attributes.splice(currentIndex, 1);
        }
      } else if (currentIndex === -1) {
        //add attribute
        attributes.push(newData);
      } else {
        //update attribute
        attributes[currentIndex] = newData;
      }
    }

    addAttribute("twitter", twitterRef.current?.value);
    addAttribute("location", locationRef.current?.value);
    addAttribute("website", websiteRef.current?.value);

    const metadata: ProfileMetadata = {
      version: MetadataVersions.one,
      metadata_id: nanoid(),
      name: nameRef.current?.value ?? null,
      bio: bioRef.current?.value ?? null,
      cover_picture,
      attributes,
    };

    try {
      await setProfileMetadata(metadata);
    } catch (err) {
      console.error(err);
    }

    setLoadingProfile(false);
  }

  async function handleProfilePictureSave() {
    if (!pfpFile || loadingProfilePicture) return;

    setLoadingProfilePicture(true);

    try {
      await setProfilePicture(pfpFile);
    } catch (err) {
      console.error(err);
    }

    setLoadingProfilePicture(false);
  }

  if (!profile) return null;

  const twitter = profile.attributes?.find((item) => item.key === "twitter");
  const website = profile.attributes?.find((item) => item.key === "website");
  const location = profile.attributes?.find((item) => item.key === "location");

  return (
    <div className="space-y-8">
      <div className="p-8 space-y-8 rounded-3xl bg-primaryContainer text-onPrimaryContainer">
        <div className="text-lg space-y-4">
          <div className="flex items-center space-x-4">
            <div className="font-bold">Profile ID:</div>
            <div>{profile.id}</div>
          </div>

          <TextField
            inputRef={nameRef}
            title="Name"
            defaultValue={profile?.name ?? ""}
          />

          <TextField
            inputRef={locationRef}
            title="Location"
            defaultValue={location?.value}
          />
          <TextField
            inputRef={websiteRef}
            title="Website"
            defaultValue={website?.value}
          />
          <TextField
            inputRef={twitterRef}
            title="Twitter"
            frontAdornment="@"
            defaultValue={twitter?.value}
          />

          <TextArea
            textAreaRef={bioRef}
            title="Bio"
            defaultValue={profile?.bio ?? ""}
          />

          <div className="space-y-4">
            <div className="text-lg font-bold">Cover</div>

            {coverUrl && (
              <div className="w-full h-40">
                <img
                  src={coverUrl}
                  alt="cover picture preview"
                  className="object-cover rounded-xl h-full w-full border"
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

        <div className="w-full flex justify-end">
          <Button
            variant="filled"
            onClick={handleProfileSave}
            loading={loadingProfile}
          >
            Save
          </Button>
        </div>
      </div>

      <div className="p-8 space-y-8 rounded-3xl bg-primaryContainer text-onPrimaryContainer">
        <div className="space-y-4 text-lg">
          <div className="font-bold">Profile Picture</div>

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
                if (file) setPfpRawFile(file);
              }}
            />
          </div>
        </div>

        <div className="w-full flex justify-end">
          <Button
            variant="filled"
            onClick={handleProfilePictureSave}
            loading={loadingProfilePicture}
            disabled={!pfpFile}
          >
            Save
          </Button>
        </div>
      </div>
    </div>
  );
}

Settings.getLayout = getSettingsLayout;
