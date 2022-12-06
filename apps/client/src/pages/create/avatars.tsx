import { getNavbarLayout } from "../../home/layouts/NavbarLayout/NavbarLayout";

export default function Avatars() {
  return (
    <div className="mx-4 flex justify-center py-8">
      <div className="max-w-content space-y-8">
        <div className="flex justify-center text-3xl font-black">Avatars</div>
      </div>
    </div>
  );
}

Avatars.getLayout = getNavbarLayout;
