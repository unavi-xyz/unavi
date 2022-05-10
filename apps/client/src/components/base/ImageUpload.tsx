import { ChangeEvent, Dispatch, SetStateAction, useState } from "react";
import { HiCamera } from "react-icons/hi";

interface Props {
  setImageFile: Dispatch<SetStateAction<File>>;
  defaultValue?: string;
}

export default function ImageUpload({
  setImageFile,
  defaultValue = "",
}: Props) {
  const [image, setImage] = useState(defaultValue);

  function handleImageChange(e: ChangeEvent<HTMLInputElement>) {
    if (!e.target.files || !e.target.files[0]) return;

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

  return (
    <div className="flex items-center space-x-4 w-full h-full">
      {image ? (
        <>
          <label
            htmlFor="image"
            className="hover:cursor-pointer h-full w-full bg-black rounded-xl"
          >
            <img
              src={image}
              alt=""
              className="object-cover rounded-xl h-full w-full hover:opacity-80 transition "
            />
          </label>
        </>
      ) : (
        <>
          <label htmlFor="image" className="hover:cursor-pointer w-full h-full">
            <span className="flex items-center justify-center space-x-4 h-full">
              <div
                className="rounded-xl  hover:bg-neutral-300 bg-neutral-200 transition
                           text-2xl flex justify-center items-center  w-full h-full"
              >
                <HiCamera />
              </div>
            </span>
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
  );
}
