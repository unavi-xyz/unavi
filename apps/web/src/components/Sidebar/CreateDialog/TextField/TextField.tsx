import { MutableRefObject } from "react";

interface Props {
  title: string;
  inputRef: MutableRefObject<HTMLInputElement>;
}

export default function TextField({ title, inputRef }: Props) {
  return (
    <div className="flex flex-col space-y-3">
      <label htmlFor={title} className="block text-xl">
        {title}
      </label>
      <input
        ref={inputRef}
        id={title}
        type="text"
        placeholder={title}
        className="border text-lg py-2 px-3 rounded leading-tight"
      />
    </div>
  );
}
