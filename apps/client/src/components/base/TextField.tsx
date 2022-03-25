import { MutableRefObject } from "react";

interface Props {
  title?: string;
  inputRef: MutableRefObject<HTMLInputElement>;
  [key: string]: any;
}

export function TextField({ title, inputRef, ...rest }: Props) {
  return (
    <div className="flex flex-col space-y-3">
      <label htmlFor={title} className="block text-lg pointer-events-none">
        {title}
      </label>
      <input
        ref={inputRef}
        id={title}
        type="text"
        className="border p-2 rounded leading-tight outline-none"
        {...rest}
      />
    </div>
  );
}
