import Image from "next/future/image";
import { useEffect, useRef, useState } from "react";

import { getSettingsLayout } from "../../src/home/layouts/SettingsLayout/SettingsLayout";
import { uploadFileToIpfs } from "../../src/lib/ipfs/uploadFileToIpfs";
import { useLens } from "../../src/lib/lens/hooks/useLens";
import { useProfileByHandle } from "../../src/lib/lens/hooks/useProfileByHandle";
import { useSetProfileImage } from "../../src/lib/lens/hooks/useSetProfileImage";
import { useSetProfileMetadata } from "../../src/lib/lens/hooks/useSetProfileMetadata";
import { createProfileMetadata } from "../../src/lib/lens/utils/createProfileMetadata";
import Button from "../../src/ui/base/Button";
import FileUpload from "../../src/ui/base/FileUpload";
import TextArea from "../../src/ui/base/TextArea";
import TextField from "../../src/ui/base/TextField";
import MetaTags from "../../src/ui/MetaTags";
import { crop } from "../../src/utils/crop";
import { getMediaURL } from "../../src/utils/getMediaURL";

export default function Settings() {
  const nameRef = useRef<HTMLInputElement>(null);
  const bioRef = useRef<HTMLTextAreaElement>(null);
  const locationRef = useRef<HTMLInputElement>(null);
  const websiteRef = useRef<HTMLInputElement>(null);
  const twitterRef = useRef<HTMLInputElement>(null);
  const hostRef = useRef<HTMLInputElement>(null);

  const [pfpRawFile, setPfpRawFile] = useState<File>();
  const [pfpFile, setPfpFile] = useState<File>();
  const [pfpUrl, setPfpUrl] = useState<string | null>(null);
  const [coverFile, setCoverFile] = useState<File>();
  const [coverUrl, setCoverUrl] = useState<string | null>(null);
  const [loadingProfile, setLoadingProfile] = useState(false);
  const [loadingProfilePicture, setLoadingProfilePicture] = useState(false);

  const { handle } = useLens();
  const profile = useProfileByHandle(handle);
  const pfpMediaUrl = getMediaURL(profile?.picture);
  const coverMediaUrl = getMediaURL(profile?.coverPicture);

  const setProfileMetadata = useSetProfileMetadata(profile?.id);
  const setProfileImage = useSetProfileImage(profile?.id);

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

    const { metadata, updateAttribute } = createProfileMetadata(profile);

    if (coverFile) {
      const uri = await uploadFileToIpfs(coverFile);
      if (uri) metadata.cover_picture = uri;
    }

    updateAttribute("location", locationRef.current?.value);
    updateAttribute("website", websiteRef.current?.value);
    updateAttribute("twitter", twitterRef.current?.value);
    updateAttribute("host", hostRef.current?.value);

    metadata.name = nameRef.current?.value ?? null;
    metadata.bio = bioRef.current?.value ?? null;

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
      await setProfileImage(pfpFile);
    } catch (err) {
      console.error(err);
    }

    setLoadingProfilePicture(false);
  }

  const twitter = profile?.attributes?.find((item) => item.key === "twitter");
  const website = profile?.attributes?.find((item) => item.key === "website");
  const location = profile?.attributes?.find((item) => item.key === "location");
  const host = profile?.attributes?.find((item) => item.key === "host");

  return (
    <>
      <MetaTags title="Settings" />

      {profile && (
        <div className="space-y-8">
          <div className="bg-primaryContainer text-onPrimaryContainer space-y-8 rounded-3xl p-8">
            <div className="space-y-4 text-lg">
              <div className="flex items-center space-x-4">
                <div className="font-bold">Profile ID:</div>
                <div>{profile.id}</div>
              </div>

              <TextField
                inputRef={nameRef}
                title="Name"
                defaultValue={profile?.name ?? ""}
              />

              <TextArea
                textAreaRef={bioRef}
                title="Bio"
                defaultValue={profile?.bio ?? ""}
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

              <div className="space-y-4">
                <div className="text-lg font-bold">Cover Image</div>

                {coverUrl && (
                  <div className="h-40 w-full">
                    <div className="relative h-full w-full object-cover">
                      <Image
                        src={coverUrl}
                        fill
                        sizes="49vw"
                        alt="cover picture preview"
                        className="h-full w-full rounded-xl object-cover"
                      />
                    </div>
                  </div>
                )}

                <FileUpload
                  title="Cover Image"
                  accept="image/*"
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) setCoverFile(file);
                  }}
                />
              </div>

              <TextField
                inputRef={hostRef}
                title="Host Server"
                help="Host server for your spaces. Can be ignored if you don't have any spaces."
                defaultValue={host?.value}
              />
            </div>

            <div className="flex w-full justify-end">
              <Button
                variant="filled"
                onClick={handleProfileSave}
                loading={loadingProfile}
              >
                Save
              </Button>
            </div>
          </div>

          <div className="bg-primaryContainer text-onPrimaryContainer space-y-8 rounded-3xl p-8">
            <div className="space-y-4 text-lg">
              <div className="font-bold">Profile Picture</div>

              {pfpUrl && (
                <div className="grid grid-cols-2 gap-x-16">
                  <div className="relative aspect-square h-full w-full">
                    <Image
                      src={pfpUrl}
                      fill
                      sizes="24vw"
                      alt="profile picture preview square"
                      className="h-full w-full rounded-xl object-cover"
                    />
                  </div>

                  <div className="relative aspect-square h-full w-full">
                    <Image
                      src={pfpUrl}
                      fill
                      sizes="24vw"
                      alt="profile picture preview circle"
                      className="h-full w-full rounded-full object-cover"
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

            <div className="flex w-full justify-end">
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
      )}
    </>
  );
}

Settings.getLayout = getSettingsLayout;
