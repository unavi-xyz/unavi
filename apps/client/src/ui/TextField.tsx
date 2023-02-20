import { RefObject, useId } from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  inputRef?: RefObject<HTMLInputElement>;
}

export default function TextField({ name, inputRef, ...rest }: Props) {
  const id = useId();

  return (
    <div className="flex flex-col space-y-1">
      {name && (
        <label htmlFor={id} className="block text-lg font-bold">
          {name}
        </label>
      )}

      <input
        ref={inputRef}
        id={id}
        name={name}
        type="text"
        className="h-full w-full rounded-md border border-neutral-200 px-3 py-2 text-neutral-900 placeholder:text-neutral-400"
        {...rest}
      />
    </div>
  );
}
