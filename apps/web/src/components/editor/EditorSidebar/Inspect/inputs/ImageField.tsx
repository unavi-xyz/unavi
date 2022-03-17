import ImageInput from "./ImageInput";

interface Props {
  title?: string;
  [key: string]: any;
}

export default function ImageField({ title, ...rest }: Props) {
  return (
    <div className="flex items-center w-full">
      <div className="w-1/4">{title}</div>
      <div className="w-full">
        <ImageInput {...rest} />
      </div>
    </div>
  );
}
