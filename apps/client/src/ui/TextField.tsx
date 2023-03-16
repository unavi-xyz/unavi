import { RefObject } from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  inputRef?: RefObject<HTMLInputElement>;
}

export default function TextField({ name, inputRef, ...rest }: Props) {
  return (
    <label>
      {name && <div className="pb-1 text-lg font-bold">{name}</div>}

      <input
        ref={inputRef}
        name={name}
        type="text"
        className="h-full w-full rounded-md border border-neutral-200 px-3 py-2 text-neutral-900 placeholder:text-neutral-400"
        {...rest}
      />
    </label>
  );
}
