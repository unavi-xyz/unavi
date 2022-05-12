import { useRef, useState } from "react";
import { utils } from "ethers";
import { nanoid } from "nanoid";

import { authenticate } from "../../../helpers/lens/authentication";
import { useEthersStore } from "../../../helpers/ethers/store";
import { useLocalSpace } from "../../../helpers/indexedDB/LocalSpace/hooks/useLocalScene";
import { useProfileByHandle } from "../../../helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../../helpers/lens/store";
import { useStudioStore } from "../../../helpers/studio/store";
import { useCreatePostTypedDataMutation } from "../../../generated/graphql";
import { LensHub__factory } from "../../../../contracts";
import { LENS_HUB_ADDRESS, WIRED_APPID } from "../../../helpers/lens/constants";
import { pollUntilIndexed } from "../../../helpers/lens/utils";

import {
  uploadFileToIpfs,
  uploadStringToIpfs,
} from "../../../helpers/ipfs/fetch";

import Button from "../../base/Button";
import FileUpload from "../../base/FileUpload";
import TextArea from "../../base/TextArea";
import TextField from "../../base/TextField";

function removeTypename(obj: any) {
  if (obj.__typename) delete obj.__typename;
  return obj;
}

export default function PublishPage() {
  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLTextAreaElement>(null);

  const id = useStudioStore((state) => state.id);
  const localSpace = useLocalSpace(id);
  const signer = useEthersStore((state) => state.signer);
  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  const image = imageFile ? URL.createObjectURL(imageFile) : localSpace?.image;

  const [, createPostTypedData] = useCreatePostTypedDataMutation();

  async function handleSubmit() {
    if (!signer || !handle || !profile || !localSpace || loading) return;

    setLoading(true);

    try {
      //authenticate
      await authenticate();

      //upload data to IPFS
      const imageURI = imageFile
        ? await uploadFileToIpfs(imageFile)
        : await uploadStringToIpfs(localSpace.image);

      const metadata = {
        version: "1.0.0",
        metadata_id: nanoid(),
        description: descriptionRef.current?.value,
        content: JSON.stringify(localSpace.scene),
        external_url: "thewired.space",
        name: nameRef.current?.value ?? "",
        attributes: [],
        image: imageURI,
        imageMimeType: undefined,
        media: [],
        animation_url: undefined,
        appId: WIRED_APPID,
      };

      const contentURI = await uploadStringToIpfs(JSON.stringify(metadata));

      //get typed data
      const { data, error } = await createPostTypedData({
        profileId: profile.id,
        contentURI,
      });

      if (error) throw new Error(error.message);
      if (!data) throw new Error("No data returned from set image");

      const typedData = data.createPostTypedData.typedData;

      //sign typed data
      const signature = await signer._signTypedData(
        removeTypename(typedData.domain),
        removeTypename(typedData.types),
        removeTypename(typedData.value)
      );
      const { v, r, s } = utils.splitSignature(signature);

      //send transaction
      const contract = LensHub__factory.connect(LENS_HUB_ADDRESS, signer);
      const tx = await contract.postWithSig({
        profileId: typedData.value.profileId,
        contentURI: typedData.value.contentURI,
        collectModule: typedData.value.collectModule,
        collectModuleData: typedData.value.collectModuleInitData,
        referenceModule: typedData.value.referenceModule,
        referenceModuleData: typedData.value.referenceModuleInitData,
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

      setLoading(false);
    } catch {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Publish</h1>
        <p className="text-lg flex justify-center">Mint a new space NFT</p>
      </div>

      <div className="w-full space-y-4">
        <div className="aspect-card rounded-2xl">
          {image && (
            <img
              src={image}
              alt="space image"
              className="w-full h-full object-cover rounded-2xl"
            />
          )}
        </div>

        <div className="w-full space-y-1">
          <div className="text-lg font-bold">Image</div>
          <FileUpload
            title="Cover Picture"
            accept="image/*"
            color="SurfaceVariant"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) setImageFile(file);
            }}
          />
        </div>

        <TextField
          inputRef={nameRef}
          title="Name"
          defaultValue={localSpace?.name}
        />
        <TextArea
          textAreaRef={descriptionRef}
          title="Description"
          defaultValue={localSpace?.description}
        />
      </div>

      <div className="flex justify-end w-full">
        <Button variant="filled" onClick={handleSubmit} loading={loading}>
          Submit
        </Button>
      </div>
    </div>
  );
}
