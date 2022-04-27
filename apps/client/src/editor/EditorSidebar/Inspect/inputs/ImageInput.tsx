import { useEffect, useState } from "react";
import { MdUpload } from "react-icons/md";
import { fileToDataUrl } from "../../../../helpers/files";

interface Props {
  title?: string;
  value?: File;
  [key: string]: any;
}

export default function ImageInput({ title, value, ...rest }: Props) {
  const [dataUrl, setDataUrl] = useState<string>();

  useEffect(() => {
    if (value) fileToDataUrl(value).then(setDataUrl);
    else setDataUrl(undefined);
  }, [value]);

  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2">{title}</div>

      <label
        htmlFor="imageInput"
        className="w-full h-6 rounded-full border relative
                   bg-cover hover:cursor-pointer hover:bg-neutral-100"
        style={{ backgroundImage: dataUrl && `url(${dataUrl})` }}
      >
        <input
          id="imageInput"
          type="file"
          accept="image/*"
          className="invisible absolute"
          {...rest}
        />

        {!value && (
          <div className="flex items-center justify-center h-full text-neutral-500">
            <MdUpload />
          </div>
        )}
      </label>
    </div>
  );
}
