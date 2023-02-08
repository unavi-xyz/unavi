type Props = React.InputHTMLAttributes<HTMLInputElement>;

export default function TextInput(props: Props) {
  return (
    <input
      {...props}
      type="text"
      className="w-full rounded border border-neutral-300 bg-neutral-50 pl-1.5 leading-snug"
    />
  );
}
