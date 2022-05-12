import { useEffect, useState } from "react";
import { useGetProfileByHandleQuery } from "../../../generated/graphql";

export function useValidateHandle(handle: string) {
  const [valid, setValid] = useState(false);
  const [error, setError] = useState<string>();

  const [{ data, error: queryError, fetching }] = useGetProfileByHandleQuery({
    variables: { handle },
    pause: handle.length < 10,
  });

  useEffect(() => {
    //min of 5 characters, we are including the .test here so its 10
    if (handle.length < 10) {
      setValid(false);
    }
  }, [handle]);

  useEffect(() => {
    if (queryError) setError(queryError.message);
  }, [queryError]);

  useEffect(() => {
    if (!data) {
      setValid(false);
      return;
    }

    if (data.profiles.items.length === 0) {
      //there is no profile with that handle
      setValid(true);
      setError(undefined);
      return;
    }

    setValid(false);
    setError("That handle is already taken");
  }, [data]);

  return { valid, error, fetching };
}
