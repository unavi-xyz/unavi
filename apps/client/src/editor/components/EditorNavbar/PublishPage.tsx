import Image from "next/future/image";
import { useRouter } from "next/router";
import { useState } from "react";

import { trpc } from "../../../auth/trpc";
import Button from "../../../ui/base/Button";
import FileInput from "../../../ui/base/FileInput";
import TextArea from "../../../ui/base/TextArea";
import TextField from "../../../ui/base/TextField";
import { useEditorStore } from "../../store";

export default function PublishPage() {
  const router = useRouter();
  const id = router.query.id as string;

  const name = useEditorStore((state) => state.name);
  const description = useEditorStore((state) => state.description);

  const { data: imageURL } = trpc.useQuery(["auth.project-image", { id }], {
    enabled: id !== undefined,
  });

  const [imageFile, setImageFile] = useState<File>();
  const [loading, setLoading] = useState(false);

  async function handlePublish() {
    setLoading(true);

    setLoading(false);
  }

  const image = imageFile ? URL.createObjectURL(imageFile) : imageURL;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="flex justify-center text-3xl">Publish Space</h1>
        <p className="flex justify-center text-lg">Mint a new space NFT</p>
      </div>

      <div className="space-y-4">
        <TextField title="Name" outline defaultValue={name} />

        <TextArea
          autoComplete="off"
          title="Description"
          outline
          defaultValue={description}
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          {image && (
            <div className="relative aspect-card h-full w-full rounded-xl bg-primaryContainer">
              <Image
                src={image}
                fill
                alt="cover picture preview"
                className="h-full w-full rounded-xl object-cover"
              />
            </div>
          )}

          <FileInput
            title="Cover Picture"
            accept="image/*"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) setImageFile(file);
            }}
          />
        </div>
      </div>

      <div className="flex justify-end">
        <Button onClick={handlePublish} variant="filled" loading={loading}>
          Submit
        </Button>
      </div>
    </div>
  );
}
