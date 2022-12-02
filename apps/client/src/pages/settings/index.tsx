import { useEffect, useRef, useState } from "react";

import { uploadFileToIpfs } from "../../client/ipfs/uploadFileToIpfs";
import { useLens } from "../../client/lens/hooks/useLens";
import { useProfileByHandle } from "../../client/lens/hooks/useProfileByHandle";
import { useSetProfileImage } from "../../client/lens/hooks/useSetProfileImage";
import { useSetProfileMetadata } from "../../client/lens/hooks/useSetProfileMetadata";
import { createProfileMetadata } from "../../client/lens/utils/createProfileMetadata";
import { getSettingsLayout } from "../../home/layouts/SettingsLayout/SettingsLayout";
import MetaTags from "../../home/MetaTags";
import Button from "../../ui/Button";
import FileInput from "../../ui/FileInput";
import TextArea from "../../ui/TextArea";
import TextField from "../../ui/TextField";
import { crop } from "../../utils/crop";
import { getMediaURL } from "../../utils/getMediaURL";

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
  const [loadingProfilePictureDelete, setLoadingProfilePictureDelete] =
    useState(false);

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
    if (!pfpFile || loadingProfilePicture || loadingProfilePictureDelete)
      return;

    setLoadingProfilePicture(true);

    try {
      await setProfileImage(pfpFile);
    } catch (err) {
      console.error(err);
    }

    setLoadingProfilePicture(false);
  }

  async function handleProfilePictureDelete() {
    if (loadingProfilePicture || loadingProfilePictureDelete) return;

    setLoadingProfilePictureDelete(true);

    try {
      await setProfileImage(null);
    } catch (err) {
      console.error(err);
    }

    setLoadingProfilePictureDelete(false);
  }

  const twitter = profile?.attributes?.find((item) => item.key === "twitter");
  const website = profile?.attributes?.find((item) => item.key === "website");
  const location = profile?.attributes?.find((item) => item.key === "location");

  return (
    <>
      <MetaTags title="Settings" />

      {profile && (
        <div className="mb-24 space-y-8">
          <div className="space-y-8 rounded-3xl bg-primaryContainer p-8 text-onPrimaryContainer">
            <div className="space-y-4 text-lg">
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
                    <img
                      src={coverUrl}
                      alt="cover image preview"
                      className="h-full w-full rounded-xl object-cover"
                      crossOrigin="anonymous"
                    />
                  </div>
                )}

                <FileInput
                  title="Cover Image"
                  accept="image/*"
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) setCoverFile(file);
                  }}
                />
              </div>
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

          <div className="space-y-8 rounded-3xl bg-primaryContainer p-8 text-onPrimaryContainer">
            <div className="space-y-4 text-lg">
              <div className="font-bold">Profile Picture</div>

              {pfpUrl && (
                <div className="grid grid-cols-2 gap-x-16">
                  <div className="relative aspect-square h-full w-full">
                    <img
                      src={pfpUrl}
                      alt="profile picture preview"
                      className="h-full w-full rounded-xl object-cover"
                      crossOrigin="anonymous"
                    />
                  </div>

                  <div className="relative aspect-square h-full w-full">
                    <img
                      src={pfpUrl}
                      alt="profile picture preview"
                      className="h-full w-full rounded-full object-cover"
                      crossOrigin="anonymous"
                    />
                  </div>
                </div>
              )}

              <div>
                <FileInput
                  title="Profile Picture"
                  accept="image/*"
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) setPfpRawFile(file);
                  }}
                />
              </div>
            </div>

            <div className="flex w-full justify-end space-x-2">
              {pfpUrl && (
                <Button
                  variant="text"
                  color="error"
                  onClick={handleProfilePictureDelete}
                  loading={loadingProfilePictureDelete}
                  disabled={loadingProfilePicture}
                >
                  Remove
                </Button>
              )}
              <Button
                variant="filled"
                onClick={handleProfilePictureSave}
                loading={loadingProfilePicture}
                disabled={!pfpFile || loadingProfilePictureDelete}
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
