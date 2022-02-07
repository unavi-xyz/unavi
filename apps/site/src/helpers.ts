function getSubdomain(hostname) {
  var regexParse = new RegExp("[a-z-0-9]{2,63}.[a-z.]{2,5}$");
  var urlParts = regexParse.exec(hostname);
  return hostname.replace(urlParts[0], "").slice(0, -1);
}

export function getAppUrl() {
  const subdomain = getSubdomain(window.location.hostname);

  //in production, direct the user to app.domain.com
  //in development, direct them to localhost
  if (subdomain === "www") {
    return `https://${window.location.hostname.replace("www", "app")}`;
  } else {
    return `http://localhost:3001`;
  }
}
