import NavbarLayout from "../src/components/layouts/NavbarLayout/NavbarLayout";

export default function Index() {
  return (
    <div className="flex justify-center py-8 mx-8">
      <div className="max-w space-y-8">
        <div
          className="flex flex-col justify-center items-center h-72 rounded-3xl
                   bg-primaryContainer text-onPrimaryContainer"
        >
          <div className="text-4xl font-black">Welcome to The Wired 👋</div>
        </div>

        <div className="h-full flex space-x-8">
          <div className="h-full w-full p-8 space-y-4">
            <div className="flex justify-center text-2xl font-black"></div>
            <div className="text-lg"></div>
          </div>

          <div className="w-full"></div>
        </div>
      </div>
    </div>
  );
}

Index.Layout = NavbarLayout;
