import { ChangeEvent, useState } from "react";

interface Props {
  title?: string;
  accept?: string;
  inputRef?: React.MutableRefObject<HTMLInputElement>;
  onChange?: (event: ChangeEvent<HTMLInputElement>) => void;
  [key: string]: any;
}

export default function FileUpload({
  title,
  accept,
  inputRef,
  onChange,
  ...rest
}: Props) {
  const [file, setFile] = useState<File>();

  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    if (onChange) onChange(e);

    const file = e.target.files?.[0];
    if (file) setFile(file);
  }

  return (
    <div className="flex flex-col space-y-1">
      <label htmlFor={title} className="block">
        <div className="group flex items-center border rounded-lg cursor-pointer">
          <div
            className="bg-neutral-100 px-3 py-2 rounded-l-lg border-r
                       group-hover:bg-neutral-200 transition-all"
          >
            Choose File
          </div>
          <div className="px-3 py-2">{file ? file.name : "No file chosen"}</div>
        </div>
      </label>

      <div>
        <input
          ref={inputRef}
          id={title}
          type="file"
          accept={accept}
          className="hidden"
          onChange={handleChange}
          {...rest}
        />
      </div>
    </div>
  );
}
