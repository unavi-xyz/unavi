import { RefObject } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  textAreaRef?: RefObject<HTMLTextAreaElement>;
}

export default function TextArea({ name, textAreaRef, ...rest }: Props) {
  return (
    <label className="block">
      {name && <div className="pb-1 text-xl font-bold">{name}</div>}

      <textarea
        ref={textAreaRef}
        name={name}
        className="h-full max-h-64 w-full rounded-xl border border-neutral-200 px-3 py-2 text-neutral-900 placeholder:text-neutral-400"
        {...rest}
      />
    </label>
  );
}
