/** Reveal the active link by moving its sidebar scrollport, never the document. */
export function revealNavigation(
  navigation: HTMLElement,
  pathname: string,
): boolean {
  const body = navigation.querySelector<HTMLElement>(".akaza-sidebar-body");
  const normalize = (path: string) => path.replace(/\/$/, "") || "/";
  const matches = (element: HTMLElement) => {
    const link = element.closest<HTMLAnchorElement>("a[href]");
    return link && normalize(link.pathname) === normalize(pathname);
  };
  let active = navigation.querySelector<HTMLElement>('[aria-current="page"]');
  // Mobile links may mount before the layer publishes its selection attributes.
  if (!active || !matches(active)) {
    active =
      [...navigation.querySelectorAll<HTMLAnchorElement>("a[href]")].find(
        matches,
      ) ?? null;
  }
  if (
    !body ||
    !active ||
    body.clientHeight === 0 ||
    !navigation.getClientRects().length
  )
    return false;
  const viewport = body.getBoundingClientRect();
  const row = active.getBoundingClientRect();
  const top = viewport.top + body.clientTop;
  const bottom = top + body.clientHeight;
  const delta =
    row.top < top ? row.top - top : Math.max(0, row.bottom - bottom);
  if (delta)
    body.scrollTo({ top: body.scrollTop + delta, behavior: "instant" });
  return true;
}
