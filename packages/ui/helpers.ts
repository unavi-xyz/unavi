function getSubdomain(hostname: string) {
  const regexParse = new RegExp("[a-z-0-9]{2,63}.[a-z.]{2,5}$");
  const urlParts = regexParse.exec(hostname);
  if (urlParts) return hostname.replace(urlParts[0], "").slice(0, -1);
}

const SUBDOMAINS = ["www", "app", "editor"];

export function getEditorUrl() {
  const subdomain = getSubdomain(window.location.hostname);

  //in production, direct the user to editor.domain.com
  //in development, direct them to localhost

  SUBDOMAINS.forEach((sub) => {
    if (subdomain === sub) {
      return `https://${window.location.hostname.replace(subdomain, "editor")}`;
    }
  });

  return `http://localhost:3002`;
}

export function getAppUrl() {
  const subdomain = getSubdomain(window.location.hostname);

  SUBDOMAINS.forEach((sub) => {
    if (subdomain === sub) {
      return `https://${window.location.hostname.replace(subdomain, "app")}`;
    }
  });

  return `http://localhost:3001`;
}

export function getHomeUrl() {
  const subdomain = getSubdomain(window.location.hostname);

  SUBDOMAINS.forEach((sub) => {
    if (subdomain === sub) {
      return `https://${window.location.hostname.replace(
        subdomain,
        "www"
      )}/home`;
    }
  });

  return `http://localhost:3000/home`;
}
