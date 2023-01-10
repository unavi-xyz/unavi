type Props = React.InputHTMLAttributes<HTMLInputElement>;

export default function NumberInput(props: Props) {
  return (
    <input
      {...props}
      type="number"
      className="w-full rounded border border-neutral-200 bg-neutral-50 pl-2 text-sm"
    />
  );
}
