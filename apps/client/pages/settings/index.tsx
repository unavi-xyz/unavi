import { utils } from "ethers";
import { nanoid } from "nanoid";
import { useEffect, useRef, useState } from "react";

import { LensHub__factory, LensPeriphery__factory } from "../../contracts";
import Button from "../../src/components/base/Button";
import FileUpload from "../../src/components/base/FileUpload";
import TextArea from "../../src/components/base/TextArea";
import TextField from "../../src/components/base/TextField";
import SettingsLayout from "../../src/components/layouts/SettingsLayout/SettingsLayout";
import {
  useCreateSetPfpTypedDataMutation,
  useCreateSetProfileMetadataTypedDataMutation,
} from "../../src/generated/graphql";
import { useEthersStore } from "../../src/helpers/ethers/store";
import {
  uploadFileToIpfs,
  uploadStringToIpfs,
} from "../../src/helpers/ipfs/fetch";
import { authenticate } from "../../src/helpers/lens/authentication";
import {
  LENS_HUB_ADDRESS,
  LENS_PERIPHERY_ADDRESS,
} from "../../src/helpers/lens/constants";
import { useMediaImage } from "../../src/helpers/lens/hooks/useMediaImage";
import { useProfileByHandle } from "../../src/helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../src/helpers/lens/store";
import {
  AttributeData,
  MetadataVersions,
  ProfileMetadata,
} from "../../src/helpers/lens/types";
import { pollUntilIndexed, removeTypename } from "../../src/helpers/lens/utils";

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
  const pfpMediaUrl = useMediaImage(profile?.picture);
  const coverMediaUrl = useMediaImage(profile?.coverPicture);

  const [, createPfpTypedData] = useCreateSetPfpTypedDataMutation();
  const [, createMetadataTypedData] =
    useCreateSetProfileMetadataTypedDataMutation();

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
    if (!signer || !profile || loadingProfile) return;
    setLoadingProfile(true);

    try {
      //authenticate
      await authenticate();

      //upload cover to ipfs
      const cover_picture: string = coverFile
        ? await uploadFileToIpfs(coverFile)
        : profile.coverPicture?.__typename === "MediaSet"
        ? profile.coverPicture.original.url
        : profile.coverPicture?.__typename === "NftImage"
        ? profile.coverPicture.uri
        : undefined;

      //create metadata
      const attributes =
        profile?.attributes?.map((attribute) => {
          const data: AttributeData = {
            key: attribute.key,
            value: attribute.value,
            traitType: attribute.traitType ?? undefined,
            displayType: (attribute.displayType as any) ?? undefined,
          };
          return data;
        }) ?? [];

      //TODO add location, website, twitter to attributes

      const metadata: ProfileMetadata = {
        version: MetadataVersions.one,
        metadata_id: nanoid(),
        name: nameRef.current?.value ?? undefined,
        bio: bioRef.current?.value ?? undefined,
        cover_picture: undefined,
        attributes: [],
      };

      //upload metdata to ipfs
      const url = await uploadStringToIpfs(JSON.stringify(metadata));

      //create typed data
      const { data, error } = await createMetadataTypedData({
        profileId: profile.id,
        metadata: url,
      });

      if (error) throw new Error(error.message);
      if (!data) throw new Error("No typed data returned");

      const typedData = data.createSetProfileMetadataTypedData.typedData;

      //sign typed data
      const signature = await signer._signTypedData(
        removeTypename(typedData.domain),
        removeTypename(typedData.types),
        removeTypename(typedData.value)
      );
      const { v, r, s } = utils.splitSignature(signature);

      //send transaction
      const contract = LensPeriphery__factory.connect(
        LENS_PERIPHERY_ADDRESS,
        signer
      );
      const tx = await contract.setProfileMetadataURIWithSig({
        user: await signer.getAddress(),
        profileId: typedData.value.profileId,
        metadata: typedData.value.metadata,
        sig: {
          v,
          r,
          s,
          deadline: typedData.value.deadline,
        },
      });

      //wait for transaction
      await tx.wait();

      //wait for indexing
      await pollUntilIndexed(tx.hash);

      //TODO update cache

      setLoadingProfile(false);
    } catch (err) {
      console.error(err);
      setLoadingProfile(false);
    }
    setLoadingProfile(false);
  }

  async function handleProfilePictureSave() {
    const signer = useEthersStore.getState().signer;
    if (!signer || !pfpFile || !profile || loadingProfilePicture) return;

    setLoadingProfilePicture(true);

    try {
      //authenticate
      await authenticate();

      //upload image to ipfs
      const url = await uploadFileToIpfs(pfpFile);

      //create typed data
      const { data, error } = await createPfpTypedData({
        profileId: profile.id,
        url,
      });

      if (error) throw new Error(error.message);
      if (!data) throw new Error("No typed data returned");

      const typedData = data.createSetProfileImageURITypedData.typedData;

      //sign typed data
      const signature = await signer._signTypedData(
        removeTypename(typedData.domain),
        removeTypename(typedData.types),
        removeTypename(typedData.value)
      );
      const { v, r, s } = utils.splitSignature(signature);

      //send transaction
      const contract = LensHub__factory.connect(LENS_HUB_ADDRESS, signer);
      const tx = await contract.setProfileImageURIWithSig({
        profileId: typedData.value.profileId,
        imageURI: typedData.value.imageURI,
        sig: {
          v,
          r,
          s,
          deadline: typedData.value.deadline,
        },
      });

      //wait for transaction
      await tx.wait();

      //wait for indexing
      await pollUntilIndexed(tx.hash);

      //TODO update cache

      setLoadingProfilePicture(false);
    } catch (err) {
      console.error(err);
      setLoadingProfilePicture(false);
    }
    setLoadingProfilePicture(false);
  }

  if (!profile) return null;

  return (
    <div className="space-y-8">
      <div className="p-8 space-y-8 rounded-3xl bg-secondaryContainer text-onSecondaryContainer">
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

          <TextField inputRef={locationRef} title="Location" />
          <TextField inputRef={websiteRef} title="Website" />
          <TextField inputRef={twitterRef} title="Twitter" />

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

      <div className="p-8 space-y-8 rounded-3xl bg-secondaryContainer text-onSecondaryContainer">
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
                if (file) setPfpFile(file);
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

Settings.Layout = SettingsLayout;
