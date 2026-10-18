// Small DOM helpers: elements built from data, never from markup strings.

export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attributes: Record<string, string | boolean | undefined> = {},
  children: (Node | string | null | undefined)[] = [],
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attributes)) {
    if (value === undefined || value === false) continue;
    if (key === 'text') node.textContent = String(value);
    else if (key === 'class') node.className = String(value);
    else if (value === true) node.setAttribute(key, '');
    else node.setAttribute(key, value);
  }
  for (const child of children) {
    if (child === null || child === undefined) continue;
    node.appendChild(typeof child === 'string' ? document.createTextNode(child) : child);
  }
  return node;
}

export function clear(node: HTMLElement): HTMLElement {
  node.textContent = '';
  return node;
}

export function button(label: string, onClick: () => void, attributes: Record<string, string | boolean | undefined> = {}): HTMLButtonElement {
  const node = el('button', { type: 'button', ...attributes, text: label });
  node.addEventListener('click', onClick);
  return node;
}

/** An inline prompt in place of a dialog: a label, a field, OK and Cancel. */
export function inlinePrompt(host: HTMLElement, label: string, initial: string, placeholder: string): Promise<string | null> {
  return new Promise((resolve) => {
    const input = el('input', { type: 'text', value: initial, placeholder, class: 'wb-prompt-input' });
    const form = el('form', { class: 'wb-prompt' }, [el('label', { text: label }), input]);
    const done = (value: string | null) => {
      form.remove();
      resolve(value);
    };
    form.appendChild(button('OK', () => done(input.value), { class: 'wb-primary' }));
    form.appendChild(button('Cancel', () => done(null)));
    form.addEventListener('submit', (event) => {
      event.preventDefault();
      done(input.value);
    });
    input.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') done(null);
    });
    host.prepend(form);
    input.focus();
    input.select();
  });
}
