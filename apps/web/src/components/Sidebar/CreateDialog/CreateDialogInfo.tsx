import { ChangeEvent, useRef, useState } from "react";
import { HiCamera } from "react-icons/hi";
import { IoMdArrowBack } from "react-icons/io";
import { useRouter } from "next/router";
import { createSpace, joinSpace } from "ceramic";

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
  const [image, setImage] = useState("");

  function handleImageChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files[0];
    const reader = new FileReader();

    reader.addEventListener(
      "load",
      () => {
        setImage(reader.result as string);
      },
      false
    );

    setImageFile(file);
    reader.readAsDataURL(file);
  }

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
        <div className="flex items-center space-x-4">
          {image ? (
            <>
              <label htmlFor="image" className="hover:cursor-pointer">
                <img
                  src={image}
                  alt=""
                  className="w-24 h-24 object-cover rounded-xl"
                />
              </label>

              <div
                onClick={() => setImage(undefined)}
                className="hover:cursor-pointer text-red-500"
              >
                <p>Delete</p>
              </div>
            </>
          ) : (
            <>
              <label htmlFor="image" className="hover:cursor-pointer">
                <span className="flex items-center justify-center space-x-4">
                  <div
                    className="w-24 h-24 rounded-xl
                               hover:bg-neutral-300 bg-neutral-200 transition-all
                                 text-2xl flex justify-center items-center duration-150"
                  >
                    <HiCamera />
                  </div>
                </span>
              </label>

              <label htmlFor="image" className="hover:cursor-pointer">
                <p>Upload</p>
              </label>
            </>
          )}

          <input
            id="image"
            type="file"
            accept="image/*"
            onChange={handleImageChange}
            className="hidden"
          />
        </div>

        <div className="flex flex-col space-y-3">
          <label htmlFor="name" className="block text-xl">
            Name
          </label>
          <input
            ref={name}
            id="name"
            type="text"
            placeholder="Name"
            className="border text-lg py-2 px-3 rounded leading-tight"
          />
        </div>

        <div className="flex flex-col space-y-3">
          <label htmlFor="description" className="block text-xl">
            Description
          </label>
          <input
            ref={description}
            id="description"
            type="text"
            placeholder="Description"
            className="border text-lg py-2 px-3 rounded leading-tight"
          />
        </div>

        <div
          onClick={handleCreate}
          className="text-2xl text-white py-2 bg-primary hover:bg-opacity-90
                     hover:cursor-pointer rounded-xl flex justify-center transition-all  duration-150"
        >
          Create
        </div>
      </form>
    </div>
  );
}
