interface Props extends React.InputHTMLAttributes<HTMLInputElement> {}

export default function TextInput(props: Props) {
  return (
    <input
      {...props}
      type="text"
      className="w-full rounded-md bg-neutral-100 pl-2.5 shadow-inner focus:outline-none"
    />
  );
}
