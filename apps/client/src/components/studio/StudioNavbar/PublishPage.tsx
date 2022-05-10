import { useState } from "react";
import Button from "../../base/Button";
import FileUpload from "../../base/FileUpload";
import TextField from "../../base/TextField";

export default function PublishPage() {
  const [imageFile, setImageFile] = useState<File>();

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Publish</h1>
        <p className="text-lg flex justify-center">Mint a new space NFT.</p>
      </div>

      <div className="flex space-x-4 items-end">
        <div className="w-full space-y-4">
          <div className="w-full space-y-1">
            <div className="text-lg font-bold">Image</div>
            <FileUpload
              title="Cover Picture"
              accept="image/*"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) setImageFile(file);
              }}
            />
          </div>

          <TextField title="Name" />
          <TextField title="Description" />
        </div>

        <div className="aspect-card w-1/2 rounded-2xl bg-neutral-200">
          {imageFile && (
            <img
              src={URL.createObjectURL(imageFile)}
              alt="space image"
              className="w-full h-full object-cover rounded-2xl"
            />
          )}
        </div>
      </div>

      <Button>Submit</Button>
    </div>
  );
}
