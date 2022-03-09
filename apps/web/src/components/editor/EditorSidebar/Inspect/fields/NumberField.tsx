interface Props {
  title?: string;
  value: number;
  [key: string]: any;
}

export default function NumberField({ title, value, ...rest }: Props) {
  return (
    <div className="flex space-x-1">
      <div className="text-neutral-500">{title}</div>

      <input
        type="number"
        value={value}
        className="border shadow-inner bg-neutral-100 outline-none px-1 rounded leading-tight w-full"
        {...rest}
      />
    </div>
  );
}
