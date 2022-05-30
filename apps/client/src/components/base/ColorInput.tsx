import { useEffect, useRef } from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  inputRef?: React.RefObject<HTMLInputElement>;
}

export default function ColorInput({ inputRef, defaultValue, ...rest }: Props) {
  const displayRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!displayRef.current || !inputRef || !inputRef.current) return;
    displayRef.current.style.backgroundColor = inputRef.current?.value;
  }, [inputRef]);

  return (
    <label className="relative">
      <div ref={displayRef} className="h-full px-6 border rounded" />
      <input
        ref={inputRef}
        onChange={(e) => {
          const color = e.target.value;
          if (displayRef.current) {
            displayRef.current.style.backgroundColor = color;
          }
        }}
        type="color"
        className="absolute bottom-0 left-0 invisible"
        {...rest}
      />
    </label>
  );
}
