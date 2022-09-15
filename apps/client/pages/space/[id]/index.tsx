import { NextPageContext } from "next";

import { getNavbarLayout } from "../../../src/home/layouts/NavbarLayout/NavbarLayout";
import {
  getSpaceLayoutProps,
  SpaceLayoutProps,
} from "../../../src/home/layouts/SpaceLayout/getSpaceLayoutProps";
import SpaceLayout from "../../../src/home/layouts/SpaceLayout/SpaceLayout";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=30");

  const props = await getSpaceLayoutProps(query.id as string);

  return {
    props,
  };
}

export default function Space(props: SpaceLayoutProps) {
  return (
    <SpaceLayout {...props}>
      <div className="space-y-2">
        <div className="text-2xl font-bold">Description</div>
        <div className="text-outline text-lg">
          {props.publication?.metadata.description}
        </div>
      </div>
    </SpaceLayout>
  );
}

Space.getLayout = getNavbarLayout;
