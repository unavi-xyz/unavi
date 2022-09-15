interface Props extends React.InputHTMLAttributes<HTMLInputElement> {}

export default function NumberInput(props: Props) {
  return (
    <input
      {...props}
      type="number"
      className="w-full rounded-md bg-neutral-100 pl-2 shadow-inner focus:outline-none"
    />
  );
}
