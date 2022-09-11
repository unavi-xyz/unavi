interface Props extends React.InputHTMLAttributes<HTMLInputElement> {}

export default function TextInput(props: Props) {
  return (
    <input
      {...props}
      type="text"
      className="bg-neutral-100 shadow-inner rounded-md focus:outline-none pl-2.5 w-full"
    />
  );
}
