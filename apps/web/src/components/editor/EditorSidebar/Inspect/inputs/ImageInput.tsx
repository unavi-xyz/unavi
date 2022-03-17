interface Props {
  title?: string;
  [key: string]: any;
}

export default function ImageInput({ title, ...rest }: Props) {
  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2">{title}</div>

      <input
        type="file"
        accept="image/*"
        className="border shadow-inner bg-neutral-100 outline-none rounded leading-tight w-full"
        {...rest}
      />
    </div>
  );
}
