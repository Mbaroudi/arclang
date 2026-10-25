// Small DOM helpers of the table view: no framework, no innerHTML.
type Child = Node | string | null | undefined | false;

export function el(tag: string, attributes: Record<string, string> = {}, children: Child | Child[] = []): HTMLElement {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
  for (const child of Array.isArray(children) ? children : [children]) {
    if (child === null || child === undefined || child === false) continue;
    node.append(typeof child === 'string' ? document.createTextNode(child) : child);
  }
  return node;
}

export function clear(node: Element): void {
  while (node.firstChild) node.removeChild(node.firstChild);
}
