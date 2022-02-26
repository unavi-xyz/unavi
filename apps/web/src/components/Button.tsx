export default function Button({ text = "", ...rest }) {
  return (
    <button
      className="text-2xl text-white py-2 bg-primary hover:bg-opacity-90 shadow-md
                 hover:cursor-pointer rounded-xl flex justify-center transition-all duration-150"
      {...rest}
    >
      {text}
    </button>
  );
}
