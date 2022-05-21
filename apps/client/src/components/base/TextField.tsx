import { ChangeEvent, RefObject } from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  title?: string;
  frontAdornment?: string;
  inputRef?: RefObject<HTMLInputElement>;
  onChange?: (e: ChangeEvent<HTMLInputElement>) => void;
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

      <div className="flex items-center border rounded-lg bg-surface text-onSurface">
        {frontAdornment && (
          <span className="pl-3 text-outline font-bold">{frontAdornment}</span>
        )}
        <input
          ref={inputRef}
          id={title}
          type="text"
          className="outline-none w-full h-full px-3 py-2 rounded-md"
          onChange={onChange}
          {...rest}
        />
      </div>
    </div>
  );
}
