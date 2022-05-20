import { useRouter } from "next/router";
import { useEffect, useRef, useState } from "react";

import Button from "../../../src/components/base/Button";
import FileUpload from "../../../src/components/base/FileUpload";
import TextArea from "../../../src/components/base/TextArea";
import TextField from "../../../src/components/base/TextField";
import LocalSpaceLayout from "../../../src/components/layouts/LocalSpaceLayout/LocalSpaceLayout";
import {
  deleteLocalSpace,
  updateLocalSpace,
} from "../../../src/helpers/indexeddb/LocalSpace/helpers";
import { useLocalSpace } from "../../../src/helpers/indexeddb/LocalSpace/hooks/useLocalSpace";
import { LocalSpace } from "../../../src/helpers/indexeddb/LocalSpace/types";

export default function Settings() {
  const router = useRouter();
  const id = router.query.id as string;

  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLTextAreaElement>(null);

  const [imageFile, setImageFile] = useState<File>();
  const [imageUrl, setImageUrl] = useState<string>();

  const localSpace = useLocalSpace(id);

  useEffect(() => {
    if (imageFile) setImageUrl(URL.createObjectURL(imageFile));
    else if (localSpace?.image)
      setImageUrl(URL.createObjectURL(localSpace.image));
    else setImageUrl(undefined);
  }, [localSpace, imageFile]);

  async function handleDelete() {
    await deleteLocalSpace(id);
    router.push("/create");
  }

  async function handleSave() {
    const changes: Partial<LocalSpace> = {
      name: nameRef.current?.value,
      description: descriptionRef.current?.value,
      image: imageFile,
      generatedImage: imageFile ? undefined : localSpace?.generatedImage,
    };

    await updateLocalSpace(id, changes);

    router.push("/create");
  }

  async function handleRemoveImage() {
    const changes: Partial<LocalSpace> = {
      image: undefined,
      generatedImage: undefined,
    };

    await updateLocalSpace(id, changes);

    router.push("/create");
  }

  return (
    <div className="space-y-4 pb-8">
      <div className="p-8 rounded-2xl bg-primaryContainer text-onPrimaryContainer space-y-4">
        <div className="text-2xl font-bold">Edit Metadata</div>

        <TextField
          inputRef={nameRef}
          title="Name"
          defaultValue={localSpace?.name ?? ""}
        />
        <TextArea
          textAreaRef={descriptionRef}
          title="Description"
          defaultValue={localSpace?.description ?? ""}
        />

        <div className="space-y-4">
          <div className="text-lg font-bold">Image</div>

          {imageUrl && (
            <div className="aspect-card h-60">
              <img
                src={imageUrl}
                alt="cover picture preview"
                className="object-cover rounded-2xl border h-full w-full"
              />
            </div>
          )}

          <FileUpload
            title="Image"
            accept="image/*"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) setImageFile(file);
            }}
          />
        </div>

        <div className="w-full flex justify-end space-x-4">
          <Button variant="outlined" squared onClick={handleRemoveImage}>
            Remove Image
          </Button>
          <Button variant="outlined" squared onClick={handleSave}>
            Save
          </Button>
        </div>
      </div>

      <div className="bg-tertiaryContainer text-onTertiaryContainer rounded-2xl p-8 space-y-4">
        <div className="text-2xl font-bold">Danger Zone</div>

        <Button
          variant="outlined"
          color="tertiary"
          squared
          onClick={handleDelete}
        >
          Delete Scene
        </Button>
      </div>
    </div>
  );
}

Settings.Layout = LocalSpaceLayout;
