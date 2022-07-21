import { useEffect, useRef } from "react";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  inputRef?: React.RefObject<HTMLInputElement>;
  defaultValue?: string;
}

export default function ColorInput({ inputRef, defaultValue, onChange, ...rest }: Props) {
  const displayRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (displayRef.current && defaultValue) {
      displayRef.current.style.backgroundColor = defaultValue;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <label className="relative">
      <div ref={displayRef} className="h-full px-6 border rounded" />
      <input
        ref={inputRef}
        onChange={(e) => {
          if (displayRef.current) {
            const color = e.target.value;
            displayRef.current.style.backgroundColor = color;
          }

          if (onChange) onChange(e);
        }}
        type="color"
        className="absolute bottom-0 left-0 invisible"
        defaultValue={defaultValue}
        {...rest}
      />
    </label>
  );
}
