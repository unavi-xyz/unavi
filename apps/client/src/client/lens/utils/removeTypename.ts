export function removeTypename(obj: any) {
  if (obj.__typename) delete obj.__typename;

  Object.values(obj).forEach((value) => {
    if (typeof value === "object") removeTypename(value);
  });

  return obj;
}
