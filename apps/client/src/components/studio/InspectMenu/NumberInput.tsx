import { useEffect, useRef } from "react";

interface Props extends React.HTMLProps<HTMLInputElement> {
  value?: number;
  updatedValue?: string;
}

export default function NumberInput({ updatedValue, ...rest }: Props) {
  const ref = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!updatedValue || !ref.current) return;
    ref.current.value = updatedValue;
  }, [updatedValue]);

  return (
    <input
      ref={ref}
      className="border outline-none pl-1 rounded-md w-full bg-surface text-onSurface shadow-inner"
      onFocus={(e) => e.currentTarget.select()}
      {...rest}
    />
  );
}
