import { useRef, useState } from "react";
import { IoMdArrowBack } from "react-icons/io";
import { useRouter } from "next/router";
import { createSpace, joinSpace } from "ceramic";

import ImageUpload from "./ImageUpload";
import TextField from "./TextField/TextField";

interface Props {
  type: string;
  back: () => void;
  close: () => void;
}

export default function CreateDialogInfo({ type, back, close }: Props) {
  const router = useRouter();

  const name = useRef<HTMLInputElement>();
  const description = useRef<HTMLInputElement>();

  const [imageFile, setImageFile] = useState<File>();

  async function handleCreate() {
    const streamId = await createSpace(
      name.current.value,
      description.current.value,
      imageFile
    );
    await joinSpace(streamId);

    close();
    router.push(`/space/${streamId}`);
  }

  return (
    <div className="flex flex-col space-y-4">
      <div className="flex items-center">
        <div onClick={back} className="absolute hover:cursor-pointer text-xl">
          <IoMdArrowBack />
        </div>
        <h1 className="text-3xl flex justify-center w-full">
          Your {type} space
        </h1>
      </div>

      <p className="text-lg flex justify-center opacity-80">
        Add some information about your space. You can change these at any time.
      </p>

      <form className="flex flex-col space-y-4">
        <ImageUpload setImageFile={setImageFile} />

        <TextField inputRef={name} title="Name" />
        <TextField inputRef={description} title="Description" />

        <span
          onClick={handleCreate}
          className="text-2xl text-white py-2 bg-primary hover:bg-opacity-90
                     hover:cursor-pointer rounded-xl flex justify-center transition-all  duration-150"
        >
          Create
        </span>
      </form>
    </div>
  );
}
