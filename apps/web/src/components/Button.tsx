export default function Button({ children, ...rest }) {
  return (
    <button
      className="text-white py-2 px-4 bg-primary hover:bg-opacity-90 shadow-md
                 hover:cursor-pointer rounded-xl flex justify-center transition-all duration-150"
      {...rest}
    >
      {children}
    </button>
  );
}
