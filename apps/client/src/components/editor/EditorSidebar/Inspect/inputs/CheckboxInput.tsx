interface Props {
  checked: boolean;
  [key: string]: any;
}

export default function CheckboxInput({ checked, ...rest }: Props) {
  return (
    <div className="flex space-x-1 w-full pr-2">
      <div className="text-neutral-500 w-2"></div>

      <input
        type="checkbox"
        checked={checked}
        className="border outline-none pl-2 rounded-full w-full"
        {...rest}
      />
    </div>
  );
}
