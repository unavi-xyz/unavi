import { ERC721Metadata } from "contracts";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { toast } from "react-hot-toast";

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
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publication/${id}/model.glb`;
}

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publication/${id}/image.jpg`;
}

interface Props {
  onClose: () => void;
}

export default function UpdatePage({ onClose }: Props) {
  const name = useEditorStore((state) => state.name);
  const description = useEditorStore((state) => state.description);
  const publicationId = useEditorStore((state) => state.publicationId);

  const { data: session } = useSession();
  const { save } = useSave();
  const router = useRouter();
  const id = router.query.id as string;

  const { data: imageURL } = trpc.project.image.useQuery(
    { id },
    {
      enabled: id !== undefined,
      refetchOnWindowFocus: false,
    }
  );

  const { data: profile } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    {
      enabled: session?.address !== undefined,
      refetchOnWindowFocus: false,
    }
  );

  const { mutateAsync: getModelUpload } = trpc.publication.getModelUpload.useMutation();
  const { mutateAsync: getImageUpload } = trpc.publication.getImageUpload.useMutation();
  const { mutateAsync: getMetadataUpload } = trpc.publication.getMetadataUpload.useMutation();

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile || !imageURL) return;
    cropImage(imageURL).then((file) => setImageFile(file));
  }, [imageFile, imageURL]);

  async function handlePublish() {
    if (loading) return;
    if (!publicationId) throw new Error("No publication id");

    async function publish() {
      async function uploadModel() {
        if (!publicationId) throw new Error("No publication id");

        const { engine } = useEditorStore.getState();
        if (!engine) throw new Error("Engine not found");

        // Export scene to glb
        const glb = await engine.modules.scene.export();
        const body = new Blob([glb], { type: "model/gltf-binary" });

        // Upload to S3
        const url = await getModelUpload({ id: publicationId });
        const response = await fetch(url, {
          method: "PUT",
          body,
          headers: {
            "Content-Type": "model/gltf-binary",
            "x-amz-acl": "public-read",
          },
        });

        if (!response.ok) throw new Error("Failed to upload model");
      }

      async function uploadImage() {
        if (!publicationId) throw new Error("No publication id");
        if (!imageFile) throw new Error("Image not found");

        // Get image
        const res = await fetch(URL.createObjectURL(imageFile));
        const body = await res.blob();

        // Upload to S3
        const url = await getImageUpload({ id: publicationId });
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

      async function uploadMetadata() {
        if (!publicationId) throw new Error("No publication id");

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
        const url = await getMetadataUpload({ id: publicationId });
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

      await save();
      await Promise.all([uploadModel(), uploadImage(), uploadMetadata()]);

      onClose();
    }

    setLoading(true);

    toast.promise(
      publish().finally(() => {
        setLoading(false);
      }),
      {
        loading: "Updating...",
        success: "Updated!",
        error: "Failed to update",
      }
    );

    setLoading(false);
  }

  const image = imageFile ? URL.createObjectURL(imageFile) : null;

  return (
    <div className="space-y-4">
      <h1 className="text-center text-3xl font-bold">Update Space</h1>

      <div className="space-y-4">
        <TextField
          onChange={(e) => {
            const value = e.target.value;
            useEditorStore.setState({ name: value });
          }}
          name="Name"
          outline
          defaultValue={name}
        />

        <TextArea
          onChange={(e) => {
            const value = e.target.value;
            useEditorStore.setState({ description: value });
          }}
          autoComplete="off"
          name="Description"
          outline
          defaultValue={description}
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          <div className="relative aspect-card h-full w-full rounded-xl bg-sky-100">
            {image && (
              <img
                src={image}
                alt="picture preview"
                className="h-full w-full rounded-xl object-cover"
              />
            )}
          </div>

          <ButtonFileInput
            name="Cover Picture"
            accept="image/*"
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
          Submit
        </Button>
      </div>
    </div>
  );
}
