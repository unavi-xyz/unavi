import { MdAddBox, MdInfo } from "react-icons/md";
import { BsFillGearFill, BsFillPeopleFill } from "react-icons/bs";
import { useRouter } from "next/router";
import { useSpace } from "ceramic";

export default function Space() {
  const router = useRouter();
  const id = router.query.id as string;

  const space = useSpace(id);

  return (
    <div>
      <div className="bg-white h-24 flex items-center px-8 space-x-16">
        <div className="flex flex-col">
          <p className="text-sm ml-[2px]">PUBLIC SPACE</p>
          <p className="text-3xl font-medium">{space?.name}</p>
        </div>

        <div className="flex space-x-4">
          <NavbarButton icon={<MdInfo className="text-[1.4rem]" />} />
          <NavbarButton icon={<BsFillPeopleFill />} />
          <NavbarButton icon={<BsFillGearFill />} />
        </div>
      </div>

      <div className="max-w-4xl px-8 py-4 flex justify-between items-center">
        <p className="text-xl font-medium">Rooms</p>

        <div
          className="flex items-center space-x-2 font-medium hover:bg-neutral-200
                     hover:cursor-pointer px-2 py-1 rounded transition-all duration-150"
        >
          <div className="text-lg">
            <MdAddBox />
          </div>
          <p className="text-md">Create</p>
        </div>
      </div>
    </div>
  );
}

function NavbarButton({ icon = <></> }) {
  return (
    <div
      className="w-14 h-14 bg-neutral-100 rounded-lg hover:cursor-pointer
               hover:bg-neutral-200 flex justify-center items-center text-xl
                 transition-all duration-150"
    >
      {icon}
    </div>
  );
}
