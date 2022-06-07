import { NextPageContext } from "next";
import Link from "next/link";

import { getNavbarLayout } from "../../../src/components/layouts/NavbarLayout/NavbarLayout";
import ProfileLayout from "../../../src/components/layouts/ProfileLayout/ProfileLayout";
import {
  ProfileLayoutProps,
  getProfileLayoutProps,
} from "../../../src/components/layouts/ProfileLayout/getProfileLayoutProps";
import SpaceCard from "../../../src/components/lens/SpaceCard";
import {
  GetPublicationsDocument,
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  Post,
} from "../../../src/generated/graphql";
import { lensClient } from "../../../src/helpers/lens/client";
import { AppId } from "../../../src/helpers/lens/types";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const handle = query.handle as string;
  const props = await getProfileLayoutProps(handle);

  if (!props.profile) return { props };

  const spacesQuery = await lensClient
    .query<GetPublicationsQuery, GetPublicationsQueryVariables>(
      GetPublicationsDocument,
      {
        profileId: props.profile.id,
        sources: [AppId.space],
      }
    )
    .toPromise();

  const spaces = spacesQuery.data?.publications.items;

  return {
    props: {
      ...props,
      spaces,
    },
  };
}

interface Props extends ProfileLayoutProps {
  spaces?: Post[];
}

export default function Spaces({ spaces, ...rest }: Props) {
  return (
    <ProfileLayout {...rest}>
      {spaces && spaces.length > 0 && (
        <div className="grid md:grid-cols-2 gap-2">
          {spaces?.map((space) => (
            <div key={space.id}>
              <Link href={`/space/${space.id}`} passHref>
                <a>
                  <SpaceCard space={space} />
                </a>
              </Link>
            </div>
          ))}
        </div>
      )}
    </ProfileLayout>
  );
}

Spaces.getLayout = getNavbarLayout;
