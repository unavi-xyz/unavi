type Props = React.InputHTMLAttributes<HTMLInputElement>;

export default function NumberInput(props: Props) {
  return (
    <input
      {...props}
      type="number"
      inputMode="numeric"
      className="w-full rounded border pl-1.5 leading-snug hover:bg-neutral-100 focus:bg-neutral-100"
    />
  );
}
