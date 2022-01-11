function getSubdomain(hostname: string) {
  const regexParse = new RegExp("[a-z-0-9]{2,63}.[a-z.]{2,5}$");
  const urlParts = regexParse.exec(hostname);
  if (urlParts) return hostname.replace(urlParts[0], "").slice(0, -1);
}

export function getEditorUrl() {
  const subdomain = getSubdomain(window.location.hostname);

  //in production, direct the user to editor.domain.com
  //in development, direct them to localhost

  if (!subdomain) return `http://localhost:3002`;

  const url = `https://${window.location.hostname}`;
  return url.replace(/^[^.]*/, "editor");
}

export function getAppUrl() {
  const subdomain = getSubdomain(window.location.hostname);

  if (!subdomain) return `http://localhost:3001`;

  const url = `https://${window.location.hostname}`;
  return url.replace(/^[^.]*/, "app");
}

export function getHomeUrl() {
  const subdomain = getSubdomain(window.location.hostname);

  if (!subdomain) return `http://localhost:3000/home`;

  const url = `https://${window.location.hostname}`;
  return `${url.replace(/^[^.]*/, "www")}/home`;
}
