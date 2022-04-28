import { ChangeEvent, RefObject } from "react";

interface Props {
  title?: string;
  frontAdornment?: string;
  inputRef?: RefObject<HTMLInputElement>;
  onChange?: (e: ChangeEvent<HTMLInputElement>) => void;
  [key: string]: any;
}

export default function TextField({
  title,
  frontAdornment,
  inputRef,
  onChange,
  ...rest
}: Props) {
  return (
    <div className="flex flex-col space-y-1">
      <label
        htmlFor={title}
        className="block text-lg font-bold pointer-events-none"
      >
        {title}
      </label>

      <div className="flex items-center border rounded-lg">
        {frontAdornment && (
          <span className="pl-2 text-neutral-500 font-bold">
            {frontAdornment}
          </span>
        )}
        <input
          ref={inputRef}
          id={title}
          type="text"
          className="outline-none w-full h-full px-3 py-2 rounded-lg"
          onChange={onChange}
          {...rest}
        />
      </div>
    </div>
  );
}
