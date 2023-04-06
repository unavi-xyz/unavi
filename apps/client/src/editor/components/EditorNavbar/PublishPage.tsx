import { Space } from "@wired-labs/gltf-extensions";
import { ATTRIBUTE_TYPES, ERC721Metadata, Space__factory, SPACE_ADDRESS } from "contracts";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import useSWR from "swr";
import { useSigner } from "wagmi";

import { GetFileDownloadResponse } from "@/app/api/projects/[id]/[file]/types";
import { publishProject } from "@/app/api/projects/[id]/publication/helper";
import { MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH } from "@/app/api/projects/constants";
import { getPublicationFileUpload } from "@/app/api/publications/[id]/files/[file]/helper";
import { linkPublication } from "@/app/api/publications/[id]/link/helper";
import { copyProjectToModel } from "@/app/api/publications/[id]/models/[modelId]/copy-project/helper";
import { getPublishedModelFileUpload } from "@/app/api/publications/[id]/models/[modelId]/files/[file]/helper";
import { createPublishedModel } from "@/app/api/publications/[id]/models/helper";
import { useEditorStore } from "@/app/editor/[id]/store";

import { useSession } from "../../../client/auth/useSession";
import { env } from "../../../env.mjs";
import { useProfileByAddress } from "../../../play/hooks/useProfileByAddress";
import { fetcher } from "../../../play/utils/fetcher";
import { Project } from "../../../server/helpers/fetchProject";
import Button from "../../../ui/Button";
import ImageInput from "../../../ui/ImageInput";
import TextArea from "../../../ui/TextArea";
import TextField from "../../../ui/TextField";
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { cdnURL, S3Path } from "../../../utils/s3Paths";
import { toHex } from "../../../utils/toHex";
import { useSave } from "../../hooks/useSave";
import { cropImage } from "../../utils/cropImage";
import { parseError } from "../../utils/parseError";

interface Props {
  project: Project;
}

export default function PublishPage({ project }: Props) {
  const router = useRouter();

  const title = useEditorStore((state) => state.title);
  const description = useEditorStore((state) => state.description);
  const image = useEditorStore((state) => state.image);

  const { data: session } = useSession();
  const { data: signer } = useSigner();
  const { save } = useSave(project.id);

  const { profile } = useProfileByAddress(session?.address);
  const { data: imageDownload } = useSWR<GetFileDownloadResponse>(
    () => `/api/projects/${project.id}/image`,
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

    if (loading) return;
    if (!signer) throw new Error("Signer not found");

    const toastId = "publish";

    async function publish() {
      const { engine } = useEditorStore.getState();

      if (!engine) throw new Error("Engine not found");
      if (!signer) throw new Error("Signer not found");
      if (!session) throw new Error("Session not found");

      // Save project
      toast.loading("Saving...", { id: toastId });
      await save();

      // Start optimizing model
      toast.loading("Creating publication...", { id: toastId });

      // Publish project
      const { id: publicationId } = await publishProject(project.id);

      // Create published model
      const modelId = await createPublishedModel(publicationId);

      const modelURL = cdnURL(S3Path.publication(publicationId).model(modelId).model);
      const imageURL = cdnURL(S3Path.publication(publicationId).model(modelId).image);

      // Update space image metadata
      const space = engine.scene.doc.getRoot().getExtension<Space>(Space.EXTENSION_NAME);
      if (!space) throw new Error("Space extension not found");

      space.setImage(imageURL);

      async function uploadModel() {
        if (!engine) throw new Error("Engine not found");

        const [url, optimizedModel] = await Promise.all([
          getPublishedModelFileUpload(publicationId, modelId, "model"),
          engine.scene.export({ optimize: true }),
        ]);

        const response = await fetch(url, {
          method: "PUT",
          body: optimizedModel,
          headers: {
            "Content-Type": "model/gltf-binary",
            "x-amz-acl": "public-read",
          },
        });

        if (!response.ok) throw new Error("Failed to upload model");

        console.info("📦 Published model size:", bytesToDisplay(optimizedModel.byteLength));
      }

      async function uploadImage() {
        if (!imageFile) throw new Error("Image not found");

        // Get image
        const res = await fetch(URL.createObjectURL(imageFile));
        const body = await res.blob();

        // Upload to S3
        const url = await getPublishedModelFileUpload(publicationId, modelId, "image");

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
        const metadata: ERC721Metadata = {
          animation_url: modelURL,
          description,
          external_url: spaceId
            ? `${env.NEXT_PUBLIC_DEPLOYED_URL}/space/${toHex(spaceId)}`
            : `${env.NEXT_PUBLIC_DEPLOYED_URL}/user/${
                profile ? toHex(profile.id) : session?.address
              }`,
          image: imageURL,
          name: title,
          attributes: [
            {
              trait_type: ATTRIBUTE_TYPES.HOST,
              value: env.NEXT_PUBLIC_DEFAULT_HOST,
            },
          ],
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

      let spaceId = project?.publication?.spaceId ?? undefined;

      if (spaceId === undefined) {
        toast.loading("Waiting for signature...", { id: toastId });

        // Mint space NFT
        const contentURI = cdnURL(S3Path.publication(publicationId).metadata);
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

      const promises: Promise<unknown>[] = [
        copyProjectToModel(publicationId, modelId, { projectId: project.id }),
        uploadModel(),
        uploadImage(),
        uploadMetadata(spaceId),
      ];

      if (spaceId !== undefined) promises.push(linkPublication(publicationId, spaceId));

      await Promise.all(promises);

      // Redirect to space if new space was created
      if (!project?.publication?.spaceId) {
        if (spaceId !== undefined) {
          // Wait for eth provider to update
          await new Promise((resolve) => setTimeout(resolve, 5000));

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
        label="Title"
        name="title"
        autoComplete="off"
        maxLength={MAX_TITLE_LENGTH}
        defaultValue={title}
        disabled={loading}
        onChange={(e) => {
          const value = e.target.value;
          useEditorStore.setState({ title: value });
        }}
      />

      <TextArea
        label="Description"
        name="description"
        autoComplete="off"
        rows={4}
        maxLength={MAX_DESCRIPTION_LENGTH}
        defaultValue={description}
        disabled={loading}
        onChange={(e) => {
          const value = e.target.value;
          useEditorStore.setState({ description: value });
        }}
      />

      <ImageInput
        name="Image"
        src={imageUrl}
        disabled={loading}
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (!file) return;
          cropImage(URL.createObjectURL(file)).then((file) => setImageFile(file));
        }}
        className="h-72 w-full"
      />

      <div className="flex justify-end">
        <Button disabled={loading} type="submit">
          Submit
        </Button>
      </div>
    </form>
  );
}
