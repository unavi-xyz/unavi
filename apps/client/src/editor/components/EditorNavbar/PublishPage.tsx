import { useConnectModal } from "@rainbow-me/rainbowkit";
import { ERC721Metadata, Space__factory, SPACE_ADDRESS } from "contracts";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import Button from "../../../ui/Button";
import ButtonFileInput from "../../../ui/ButtonFileInput";
import TextArea from "../../../ui/TextArea";
import TextField from "../../../ui/TextField";
import { numberToHexDisplay } from "../../../utils/numberToHexDisplay";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import { cropImage } from "../../utils/cropImage";

function cdnModelURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/model.glb`;
}

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/image.jpg`;
}

function cdnMetadataURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/published/${id}/metadata.json`;
}

export default function PublishPage() {
  const router = useRouter();
  const id = router.query.id as string;

  const name = useEditorStore((state) => state.name);
  const description = useEditorStore((state) => state.description);

  const { data: session } = useSession();
  const { data: signer } = useSigner();
  const { openConnectModal } = useConnectModal();
  const { save } = useSave();

  const { data: imageURL } = trpc.project.image.useQuery(
    { id },
    {
      enabled: id !== undefined,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    }
  );

  const { data: profile } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    {
      enabled: session?.address !== undefined,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    }
  );

  const { mutateAsync: createModelUploadUrl } = trpc.publication.modelUploadURL.useMutation();
  const { mutateAsync: createImageUploadUrl } = trpc.publication.imageUploadURL.useMutation();
  const { mutateAsync: createMetadataUploadUrl } = trpc.publication.metadataUploadURL.useMutation();
  const { mutateAsync: createPublication } = trpc.publication.create.useMutation();
  // const { mutateAsync: linkPublication } = trpc.publication.link.useMutation();

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile || !imageURL) return;
    cropImage(imageURL).then((file) => setImageFile(file));
  }, [imageFile, imageURL]);

  function handlePublish() {
    if (loading) return;

    if (!signer) {
      if (openConnectModal) openConnectModal();
      return;
    }

    setLoading(true);
    const promises: Promise<void>[] = [];

    async function publish() {
      if (!signer) throw new Error("Signer not found");

      // Save project
      await save();

      // Create database publication
      const publicationId = await createPublication({ type: "SPACE" });

      // Export scene and upload to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            const { engine } = useEditorStore.getState();
            if (!engine) throw new Error("Engine not found");

            // Export scene to glb
            const glb = await engine.export();
            const body = new Blob([glb], { type: "model/gltf-binary" });

            // Upload to S3
            const url = await createModelUploadUrl({ id: publicationId });
            const response = await fetch(url, {
              method: "PUT",
              body,
              headers: {
                "Content-Type": "model/gltf-binary",
                "x-amz-acl": "public-read",
              },
            });

            if (!response.ok) reject();
            else resolve();
          }

          upload();
        })
      );

      // Upload image to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            if (!imageFile) {
              reject();
              return;
            }

            // Get image
            const res = await fetch(URL.createObjectURL(imageFile));
            const body = await res.blob();

            // Upload to S3
            const url = await createImageUploadUrl({ id: publicationId });
            const response = await fetch(url, {
              method: "PUT",
              body,
              headers: {
                "Content-Type": "image/jpeg",
                "x-amz-acl": "public-read",
              },
            });

            if (!response.ok) reject();
            else resolve();
          }

          upload();
        })
      );

      // Upload metadata to S3
      promises.push(
        new Promise((resolve, reject) => {
          async function upload() {
            const modelURL = cdnModelURL(publicationId);
            const imageURL = cdnImageURL(publicationId);

            const metadata: ERC721Metadata = {
              animation_url: modelURL,
              description,
              external_url: `https://thewired.space/user/${
                profile ? numberToHexDisplay(profile.id) : session?.address
              }`,
              image: imageURL,
              name,
            };

            // Upload to S3
            const url = await createMetadataUploadUrl({ id: publicationId });
            const response = await fetch(url, {
              method: "PUT",
              body: JSON.stringify(metadata),
              headers: {
                "Content-Type": "application/json",
                "x-amz-acl": "public-read",
              },
            });

            if (response.ok) resolve();
            else reject();
          }

          upload();
        })
      );

      await Promise.all(promises);

      const contentURI = cdnMetadataURL(publicationId);

      // Mint space NFT
      const contract = Space__factory.connect(SPACE_ADDRESS, signer);

      const tx = await contract.mintWithTokenURI(contentURI);

      await tx.wait();
    }

    toast.promise(
      publish().finally(() => {
        setLoading(false);
      }),
      {
        loading: "Publishing...",
        success: "Published!",
        error: "Failed to publish",
      }
    );
  }

  const image = imageFile ? URL.createObjectURL(imageFile) : null;

  return (
    <div className="space-y-6">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="flex justify-center text-3xl font-bold">Publish Space</h1>
      </div>

      <div className="space-y-4">
        <TextField
          name="Name"
          onChange={(e) => {
            const value = e.target.value;
            useEditorStore.setState({ name: value });
          }}
          outline
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
          outline
          rows={4}
          defaultValue={description}
          disabled={loading}
        />

        <div className="space-y-2">
          <div className="text-lg font-bold">Image</div>

          <div className="relative aspect-card h-full w-full rounded-xl bg-sky-100">
            {image && (
              <img src={image} alt="picture preview" className="h-full w-full rounded-xl" />
            )}
          </div>

          <ButtonFileInput
            name="Cover Picture"
            accept="image/*"
            disabled={loading}
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (!file) return;

              cropImage(URL.createObjectURL(file)).then((file) => setImageFile(file));
            }}
          >
            Upload Image
          </ButtonFileInput>
        </div>
      </div>

      <div className="flex justify-end">
        <Button onClick={handlePublish} variant="filled" disabled={loading}>
          {signer ? "Submit" : "Sign In"}
        </Button>
      </div>
    </div>
  );
}
