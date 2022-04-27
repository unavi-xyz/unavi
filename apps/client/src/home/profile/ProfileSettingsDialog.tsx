import { Dispatch, SetStateAction, useContext, useRef, useState } from "react";
import { utils } from "ethers";
import { nanoid } from "nanoid";
import omitDeep from "omit-deep";

import { uploadFileToIpfs, ImageSources, IpfsContext } from "ceramic";

import { Button, Dialog, ImageUpload, TextField } from "../../components/base";
import { useProfileByHandle } from "../../helpers/lens/hooks/useProfileByHandle";
import { authenticate } from "../../helpers/lens/authentication";
import { ProfileMetadata } from "../../helpers/lens/types";
import { useLensStore } from "../../helpers/lens/store";
import { apolloClient } from "../../helpers/lens/apollo";
import { pollUntilIndexed } from "../../helpers/lens/pollUntilIndexed";
import { LENS_PERIPHERY_ADDRESS } from "../../helpers/lens/contracts";
import { LensPeriphery__factory } from "../../../contracts";
import {
  GET_PROFILE_BY_HANDLE,
  SET_PROFILE_METADATA,
} from "../../helpers/lens/queries";
import {
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
  SetProfileMetadataMutation,
  SetProfileMetadataMutationVariables,
} from "../../generated/graphql";
import { useEthersStore } from "../../helpers/ethers/store";

interface Props {
  handle: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function ProfileSettingsDialog({ handle, open, setOpen }: Props) {
  const nameRef = useRef<HTMLInputElement>();
  const bioRef = useRef<HTMLTextAreaElement>();

  const { ipfs } = useContext(IpfsContext);

  const { data } = useProfileByHandle(handle);
  const profile = data?.profiles.items[0];
  const profilePictureUrl = profile?.metadata?.profilePictureUrl;

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  async function handleSave() {
    if (loading) return;
    setLoading(true);

    const name = nameRef.current.value;
    const bio = bioRef.current.value;

    try {
      await authenticate();

      //create metadata
      const metadata: ProfileMetadata = {
        version: "1.0.0",
        metadata_id: nanoid(),
        social: [],
        attributes: [],
        cover_picture: "",
        location: "",
        name,
        bio,
      };

      //upload to ipfs
      const stringified = JSON.stringify(metadata);
      const cid = await ipfs.add(stringified);
      const hash = `ipfs://${cid.path}`;

      //generate typed data
      const { data } = await apolloClient.mutate<
        SetProfileMetadataMutation,
        SetProfileMetadataMutationVariables
      >({
        mutation: SET_PROFILE_METADATA,
        variables: { profileId: profile.id, metadata: hash },
      });
      const typedData = data.createSetProfileMetadataTypedData.typedData;

      //sign typed data
      const signer = useEthersStore.getState().signer;
      const signature = await signer._signTypedData(
        omitDeep(typedData.domain, "__typename"),
        omitDeep(typedData.types, "__typename"),
        omitDeep(typedData.value, "__typename")
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

      //update profile cache with new metadata
      apolloClient.cache.updateQuery<
        GetProfileByHandleQuery,
        GetProfileByHandleQueryVariables
      >({ query: GET_PROFILE_BY_HANDLE, variables: { handle } }, (data) => ({
        profiles: {
          items: [{ ...profile, ...metadata }],
          __typename: "PaginatedProfileResult",
        },
      }));

      setLoading(false);
      setOpen(false);
      return;
    } catch {}

    setLoading(false);
    return;
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Edit Profile</h1>

        {/* <div className="h-32 w-32">
          <ImageUpload
            setImageFile={setImageFile}
            defaultValue={profilePictureUrl}
          />
        </div> */}

        <div className="space-y-4">
          <TextField
            title="Name"
            inputRef={nameRef}
            defaultValue={profile?.name}
          />

          <div className="flex flex-col space-y-3">
            <label className="block text-lg pointer-events-none">Bio</label>
            <textarea
              ref={bioRef}
              defaultValue={profile?.bio}
              maxLength={420}
              rows={8}
              className="w-full border p-2 leading-tight rounded"
            />
          </div>
        </div>

        <Button loading={loading} onClick={handleSave}>
          Save
        </Button>
      </div>
    </Dialog>
  );
}
