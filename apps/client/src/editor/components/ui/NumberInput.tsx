type Props = React.InputHTMLAttributes<HTMLInputElement>;

export default function NumberInput(props: Props) {
  return (
    <input
      {...props}
      type="number"
      className="w-full rounded border border-neutral-300 bg-neutral-50 pl-1 leading-snug"
    />
  );
}
