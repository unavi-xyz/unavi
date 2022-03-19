interface Props {
  title?: string;
  value: number;
  [key: string]: any;
}

export default function NumberInput({ title, value, ...rest }: Props) {
  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2">{title}</div>

      <input
        type="number"
        value={value}
        className="border outline-none pl-2 rounded-full w-full"
        {...rest}
      />
    </div>
  );
}
