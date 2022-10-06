import { ChangeEvent, useId, useState } from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  displayName?: string | null;
  inputRef?: React.MutableRefObject<HTMLInputElement>;
}

export default function FileInput({
  displayName,
  inputRef,
  onChange,
  ...rest
}: Props) {
  const id = useId();

  const [file, setFile] = useState<File>();

  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    if (onChange) onChange(e);

    const file = e.target.files?.[0];
    if (file) setFile(file);
  }

  return (
    <div className="flex flex-col space-y-1">
      <label
        htmlFor={id}
        className="group block cursor-pointer rounded-md transition hover:shadow active:shadow-md"
      >
        <div className="flex items-center">
          <div className="select-none rounded-l-lg px-3 py-2">Choose File</div>
          <div className="select-none break-all px-3 py-2">
            {displayName ?? (file ? file.name : "No file chosen")}
          </div>
        </div>
      </label>

      <div>
        <input
          ref={inputRef}
          id={id}
          type="file"
          className="hidden"
          onChange={handleChange}
          {...rest}
        />
      </div>
    </div>
  );
}
