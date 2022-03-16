import { ChangeEvent } from "react";
import ImageInput from "./ImageInput";

interface Props {
  title?: string;
  value: string;
  onChange: (value: string) => void;
}

export default function ImageField({ title, value, onChange }: Props) {
  function handleChange(e: ChangeEvent<HTMLInputElement>) {
    if (!e.target.files || !e.target.files[0]) return;

    const file = e.target.files[0];
    const reader = new FileReader();

    reader.addEventListener(
      "load",
      () => {
        onChange(reader.result as string);
      },
      false
    );

    reader.readAsDataURL(file);
  }

  return (
    <div className="flex items-center w-full">
      <div className="w-1/4">{title}</div>
      <div className="w-full">
        <ImageInput value={value} onChange={handleChange} />
      </div>
    </div>
  );
}
