interface Props extends React.HTMLProps<HTMLInputElement> {}

export default function NumberInput({ ...props }: Props) {
  return (
    <input
      {...props}
      type="number"
      className="bg-neutral-100 shadow-inner max-w-min min-w-0 rounded focus:outline-none pl-2"
    />
  );
}
