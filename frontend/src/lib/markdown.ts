import DOMPurify from 'dompurify';
import { marked } from 'marked';

/** Renders user markdown (node notes) to sanitized HTML. */
export function renderMarkdown(source: string): string {
	return DOMPurify.sanitize(marked.parse(source, { async: false, gfm: true, breaks: true }));
}
