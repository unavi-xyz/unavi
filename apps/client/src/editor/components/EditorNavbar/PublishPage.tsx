import { ERC721Metadata, Space__factory, SPACE_ADDRESS } from "contracts";
import { useRouter, useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import useSWR from "swr";
import { useSigner } from "wagmi";

import { GetFileDownloadResponse } from "../../../../app/api/projects/[id]/[file]/types";
import { publishProject } from "../../../../app/api/projects/[id]/publication/helper";
import { getPublicationFileUpload } from "../../../../app/api/publications/[id]/[file]/upload/helper";
import { linkPublication } from "../../../../app/api/publications/[id]/link/helper";
import { useSession } from "../../../client/auth/useSession";
import { env } from "../../../env/client.mjs";
import { useProfileByAddress } from "../../../play/hooks/useProfileByAddress";
import { fetcher } from "../../../play/utils/fetcher";
import { Project } from "../../../server/helpers/fetchProject";
import Button from "../../../ui/Button";
import ImageInput from "../../../ui/ImageInput";
import TextArea from "../../../ui/TextArea";
import TextField from "../../../ui/TextField";
import { toHex } from "../../../utils/toHex";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import { cropImage } from "../../utils/cropImage";

function cdnModelURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publications/${id}/model.glb`;
}

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publications/${id}/image.jpg`;
}

function cdnMetadataURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publications/${id}/metadata.json`;
}

export default function PublishPage() {
  const router = useRouter();
  const params = useSearchParams();
  const id = params?.get("id");

  const name = useEditorStore((state) => state.name);
  const description = useEditorStore((state) => state.description);

  const { data: session } = useSession();
  const { data: signer } = useSigner();
  const { save } = useSave();

  const { profile } = useProfileByAddress(session?.address);
  const { data: project } = useSWR<Project | null>(`/api/projects/${id}`, fetcher);
  const { data: imageDownload } = useSWR<GetFileDownloadResponse>(
    `/api/projects/${id}/image`,
    fetcher
  );

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile || !imageDownload) return;
    cropImage(imageDownload.url).then((file) => setImageFile(file));
  }, [imageFile, imageDownload]);

  function handlePublish() {
    if (loading || !signer || !id) return;

    async function publish() {
      if (!signer) throw new Error("Signer not found");
      if (!session) throw new Error("Session not found");
      if (!id) throw new Error("Project ID not found");

      await save();

      const publicationId = project?.Publication?.id ?? (await publishProject(id));

      async function uploadImage() {
        if (!imageFile) throw new Error("Image not found");
        if (!id) throw new Error("Project ID not found");

        // Get image
        const res = await fetch(URL.createObjectURL(imageFile));
        const body = await res.blob();

        // Upload to S3
        const url = await getPublicationFileUpload(id, "image");

        const response = await fetch(url, {
          method: "PUT",
          body,
          headers: {
            "Content-Type": "image/jpeg",
            "x-amz-acl": "public-read",
          },
        });

        if (!response.ok) throw new Error("Failed to upload image");
      }

      async function uploadMetadata(spaceId: number | undefined) {
        if (!id) throw new Error("Project ID not found");

        const modelURL = cdnModelURL(publicationId);
        const imageURL = cdnImageURL(publicationId);

        const metadata: ERC721Metadata = {
          animation_url: modelURL,
          description,
          external_url: spaceId
            ? `https://thewired.space/space/${toHex(spaceId)}`
            : `https://thewired.space/user/${profile ? toHex(profile.id) : session?.address}`,
          image: imageURL,
          name,
        };

        // Upload to S3
        const url = await getPublicationFileUpload(id, "model");

        const response = await fetch(url, {
          method: "PUT",
          body: JSON.stringify(metadata),
          headers: {
            "Content-Type": "application/json",
            "x-amz-acl": "public-read",
          },
        });

        if (!response.ok) throw new Error("Failed to upload metadata");
      }

      // Mint space NFT
      const contentURI = cdnMetadataURL(publicationId);
      const contract = Space__factory.connect(SPACE_ADDRESS, signer);
      const tx = await contract.mintWithTokenURI(contentURI);
      await tx.wait();

      // Get space ID
      // Loop backwards through all spaces until we find the matching content URI
      const count = (await contract.count()).toNumber();
      let spaceId = project?.Publication?.spaceId ?? undefined;
      let i = 0;

      while (spaceId === undefined && i < count) {
        i++;
        const tokenId = count - i;

        const owner = await contract.ownerOf(tokenId);
        if (owner !== session.address) continue;

        const uri = await contract.tokenURI(tokenId);
        if (uri !== contentURI) continue;

        spaceId = tokenId;
      }

      const promises: Promise<unknown>[] = [uploadImage(), uploadMetadata(spaceId)];

      if (spaceId !== undefined) promises.push(linkPublication(publicationId, spaceId));

      await Promise.all(promises);
      await new Promise((resolve) => setTimeout(resolve, 3000));

      if (spaceId !== undefined) {
        // Redirect to space
        router.push(`/space/${toHex(spaceId)}`);
      } else {
        // Redirect to profile
        router.push(`/user/${profile ? toHex(profile.id) : session?.address}`);
      }
    }

    setLoading(true);

    toast
      .promise(publish(), {
        loading: "Publishing...",
        success: "Published!",
        error: "Failed to publish",
      })
      .catch((err) => console.error(err))
      .finally(() => setLoading(false));
  }

  const image = imageFile ? URL.createObjectURL(imageFile) : undefined;

  return (
    <div className="space-y-4">
      <TextField
        name="Name"
        onChange={(e) => {
          const value = e.target.value;
          useEditorStore.setState({ name: value });
        }}
        autoComplete="off"
        defaultValue={name}
        disabled={loading}
      />

      <TextArea
        name="Description"
        onChange={(e) => {
          const value = e.target.value;
          useEditorStore.setState({ description: value });
        }}
        autoComplete="off"
        rows={4}
        defaultValue={description}
        disabled={loading}
      />

      <div className="space-y-2">
        <div className="text-lg font-bold">Image</div>

        <ImageInput
          src={image}
          disabled={loading}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (!file) return;
            cropImage(URL.createObjectURL(file)).then((file) => setImageFile(file));
          }}
        />
      </div>

      <div className="flex justify-end">
        <Button disabled={loading} onClick={handlePublish}>
          Submit
        </Button>
      </div>
    </div>
  );
}
