interface Props extends React.InputHTMLAttributes<HTMLInputElement> {}

export default function NumberInput(props: Props) {
  return (
    <input
      {...props}
      type="number"
      className="bg-neutral-100 shadow-inner rounded-md focus:outline-none pl-2 w-full"
    />
  );
}
