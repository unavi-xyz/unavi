import { ChangeEvent, useId, useState } from "react";

import Button from "./Button";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  displayText?: string | null;
  inputRef?: React.MutableRefObject<HTMLInputElement>;
}

export default function ButtonFileInput({
  displayText,
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
      <label htmlFor={id}>
        <Button labelId={id} fullWidth rounded="large">
          {displayText ?? file?.name}
        </Button>
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
