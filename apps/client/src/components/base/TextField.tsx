import { MutableRefObject } from "react";

interface Props {
  title?: string;
  frontAdornment?: string;
  inputRef?: MutableRefObject<HTMLInputElement>;
  [key: string]: any;
}

export function TextField({ title, inputRef, frontAdornment, ...rest }: Props) {
  return (
    <div className="flex flex-col space-y-3">
      <label htmlFor={title} className="block text-lg pointer-events-none">
        {title}
      </label>

      <div className="flex items-center border rounded leading-tight">
        {frontAdornment && (
          <span className="pl-2 text-neutral-500">{frontAdornment}</span>
        )}
        <input
          ref={inputRef}
          id={title}
          type="text"
          className="outline-none w-full h-full p-2"
          {...rest}
        />
      </div>
    </div>
  );
}
