// The documents `arclang document` writes: its formats and its arguments.

export type DocumentFormat = 'docx' | 'html' | 'md';

export const DOCUMENT_FORMATS: { id: DocumentFormat; label: string; detail: string }[] = [
  { id: 'docx', label: 'Word', detail: 'A .docx with headings, tables and lists, to review and annotate.' },
  { id: 'html', label: 'HTML', detail: 'One page that reads on its own, nothing loaded from elsewhere.' },
  { id: 'md', label: 'Markdown', detail: 'Text to keep beside the model in git, rendered by GitHub or GitLab.' },
];

/** `arclang document <model> -o <target> [--template <file>]`. */
export function documentArgs(model: string, target: string, template: string | null): string[] {
  return ['document', model, '-o', target, ...(template ? ['--template', template] : [])];
}
