interface Props {
  title?: string;
  value: string;
  [key: string]: any;
}

export default function ImageInput({ title, value, ...rest }: Props) {
  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2">{title}</div>

      <input
        type="file"
        accept="image/*"
        // value={value}
        className="border shadow-inner bg-neutral-100 outline-none rounded leading-tight w-full"
        {...rest}
      />
    </div>
  );
}
