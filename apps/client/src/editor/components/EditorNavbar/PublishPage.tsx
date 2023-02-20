import { ERC721Metadata, Space__factory, SPACE_ADDRESS } from "contracts";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { toast } from "react-hot-toast";
import { useSigner } from "wagmi";

import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import Button from "../../../ui/Button";
import ImageInput from "../../../ui/ImageInput";
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

function cdnMetadataURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publication/${id}/metadata.json`;
}

export default function PublishPage() {
  const router = useRouter();
  const id = router.query.id as string;

  const name = useEditorStore((state) => state.name);
  const description = useEditorStore((state) => state.description);

  const { data: session } = useSession();
  const { data: signer } = useSigner();
  const { save } = useSave();
  const utils = trpc.useContext();

  const { data: imageURL } = trpc.project.image.useQuery(
    { id },
    { enabled: id !== undefined, refetchOnWindowFocus: false }
  );

  const { data: profile } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined, refetchOnWindowFocus: false }
  );

  const { mutateAsync: getImageUpload } = trpc.publication.getImageUpload.useMutation();
  const { mutateAsync: getMetadataUpload } = trpc.publication.getMetadataUpload.useMutation();
  const { mutateAsync: publish } = trpc.project.publish.useMutation();
  const { mutateAsync: linkPublication } = trpc.publication.link.useMutation();

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (imageFile || !imageURL) return;
    cropImage(imageURL).then((file) => setImageFile(file));
  }, [imageFile, imageURL]);

  function handlePublish() {
    if (loading || !signer) return;

    async function publishProject() {
      if (!signer) throw new Error("Signer not found");
      if (!session) throw new Error("Session not found");

      await save();
      const publicationId = await publish({ id });

      async function uploadImage() {
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
        const modelURL = cdnModelURL(publicationId);
        const imageURL = cdnImageURL(publicationId);

        const metadata: ERC721Metadata = {
          animation_url: modelURL,
          description,
          external_url: spaceId
            ? `https://thewired.space/space/${numberToHexDisplay(spaceId)}`
            : `https://thewired.space/user/${
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

      // Mint space NFT
      const contentURI = cdnMetadataURL(publicationId);
      const contract = Space__factory.connect(SPACE_ADDRESS, signer);
      const tx = await contract.mintWithTokenURI(contentURI);
      await tx.wait();

      // Get space ID
      // Loop backwards through all spaces until we find the matching content URI
      const count = (await contract.count()).toNumber();
      let spaceId = 0;
      let i = 0;

      while (spaceId === 0) {
        i++;
        const tokenId = count - i;

        const owner = await contract.ownerOf(tokenId);
        if (owner !== session.address) continue;

        const uri = await contract.tokenURI(tokenId);
        if (uri !== contentURI) continue;

        spaceId = tokenId;
      }

      await Promise.all([
        uploadImage(),
        uploadMetadata(),
        linkPublication({ publicationId, spaceId }),
        utils.space.latest.invalidate({ owner: session.address }),
      ]);

      // Redirect to space
      await new Promise((resolve) => setTimeout(resolve, 5000));
      router.push(`/space/${numberToHexDisplay(spaceId)}`);
    }

    setLoading(true);

    toast
      .promise(publishProject(), {
        loading: "Publishing...",
        success: "Published!",
        error: "Failed to publish",
      })
      .catch((err) => console.error(err))
      .finally(() => {
        setLoading(false);
      });
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
