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
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { toHex } from "../../../utils/toHex";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import { cropImage } from "../../utils/cropImage";
import { parseError } from "../../utils/parseError";

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
  const image = useEditorStore((state) => state.image);

  const { data: session } = useSession();
  const { data: signer } = useSigner();
  const { save } = useSave();

  const { profile } = useProfileByAddress(session?.address);
  const { data: project } = useSWR<Project | null>(
    () => (id ? `/api/projects/${id}` : null),
    fetcher,
    {
      revalidateOnFocus: false,
      revalidateOnReconnect: false,
    }
  );
  const { data: imageDownload } = useSWR<GetFileDownloadResponse>(
    () => (id ? `/api/projects/${id}/image` : null),
    fetcher,
    { revalidateOnFocus: false, revalidateOnReconnect: false }
  );

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile) return;
    if (image) cropImage(image).then(setImageFile);
    else if (imageDownload) cropImage(imageDownload.url).then(setImageFile);
  }, [imageFile, imageDownload, image]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (loading || !signer || !id) return;

    const toastId = "publish";

    async function publish() {
      const { engine } = useEditorStore.getState();

      if (!engine) throw new Error("Engine not found");
      if (!signer) throw new Error("Signer not found");
      if (!session) throw new Error("Session not found");
      if (!id) throw new Error("Project ID not found");

      toast.loading("Saving...", { id: toastId });
      await save();

      toast.loading("Optimizing model...", { id: toastId });

      let publicationId: string;
      let modelSize: number;

      // Try to optimize model
      // If it fails, publish without optimization
      try {
        const res = await publishProject(id);
        publicationId = res.id;
        modelSize = res.modelSize;
      } catch (err) {
        console.warn(err);

        toast.error("Failed to optimize model");
        toast.loading("Publishing model...", { id: toastId });

        const res = await publishProject(id, { optimize: false });
        publicationId = res.id;
        modelSize = res.modelSize;
      }

      console.info("📦 Published model size:", bytesToDisplay(modelSize));

      async function uploadImage() {
        if (!imageFile) throw new Error("Image not found");
        if (!id) throw new Error("Project ID not found");

        // Get image
        const res = await fetch(URL.createObjectURL(imageFile));
        const body = await res.blob();

        // Upload to S3
        const url = await getPublicationFileUpload(publicationId, "image");

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
        const url = await getPublicationFileUpload(publicationId, "metadata");

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

      let spaceId = project?.Publication?.spaceId ?? undefined;

      if (spaceId === undefined) {
        toast.loading("Waiting for signature...", { id: toastId });

        // Mint space NFT
        const contentURI = cdnMetadataURL(publicationId);
        const contract = Space__factory.connect(SPACE_ADDRESS, signer);
        const tx = await contract.mintWithTokenURI(contentURI);

        toast.loading("Minting space...", { id: toastId });

        await tx.wait();

        // Get space ID
        // Loop backwards through all spaces until we find the matching content URI
        const count = (await contract.count()).toNumber();
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
      }

      toast.loading("Uploading metadata...", { id: toastId });
      const promises: Promise<unknown>[] = [uploadImage(), uploadMetadata(spaceId)];

      if (spaceId !== undefined) promises.push(linkPublication(publicationId, spaceId));

      await Promise.all(promises);

      // Redirect to space if new space was created
      if (!project?.Publication?.spaceId) {
        await new Promise((resolve) => setTimeout(resolve, 3000));

        if (spaceId !== undefined) {
          // Redirect to space
          router.push(`/space/${toHex(spaceId)}`);
        } else {
          // Redirect to profile
          router.push(`/user/${profile ? toHex(profile.id) : session?.address}`);
        }
      }
    }

    setLoading(true);

    try {
      await publish();
      toast.success("Published!", { id: toastId });
    } catch (err) {
      toast.error(parseError(err, "Failed to publish."), { id: toastId });
      console.error(err);
    }

    setLoading(false);
  }

  const imageUrl = imageFile ? URL.createObjectURL(imageFile) : undefined;

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
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
          src={imageUrl}
          disabled={loading}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (!file) return;
            cropImage(URL.createObjectURL(file)).then((file) => setImageFile(file));
          }}
          className="h-72 w-full"
        />
      </div>

      <div className="flex justify-end">
        <Button disabled={loading} type="submit">
          Submit
        </Button>
      </div>
    </form>
  );
}
