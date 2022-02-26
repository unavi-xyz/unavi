import { ChangeEvent, Dispatch, SetStateAction, useState } from "react";
import { HiCamera } from "react-icons/hi";

interface Props {
  setImageFile: Dispatch<SetStateAction<File>>;
}

export default function ImageUpload({ setImageFile }: Props) {
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

  return (
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
            <span>Delete</span>
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
            <span>Upload</span>
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
