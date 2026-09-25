import { goto } from '$app/navigation';
import { page } from '$app/state';

/** Current URL with one query param set (or removed when `value` is null). */
export function withParam(name: string, value: string | null): string {
	const params = new URLSearchParams(page.url.search);
	if (value === null) params.delete(name);
	else params.set(name, value);
	const query = params.toString();
	return `${page.url.pathname}${query ? `?${query}` : ''}`;
}

/** Opens the node detail slide-over by deep link (`?node=<id>`) on the current page. */
export function openNode(nodeId: string) {
	goto(withParam('node', nodeId), { noScroll: true, keepFocus: true });
}

export function closeNode() {
	goto(withParam('node', null), { noScroll: true, keepFocus: true });
}

/**
 * The `?next=` path to land on after signing in. Same-origin paths only -
 * `//evil.example` and `/\evil.example` are protocol-relative to browsers,
 * so a crafted login link could otherwise bounce the user off-site.
 */
export function nextPath(search: string, fallback = '/dashboard'): string {
	const next = new URLSearchParams(search).get('next');
	const sameOrigin = next?.startsWith('/') && !next.startsWith('//') && !next.startsWith('/\\');
	return sameOrigin ? next! : fallback;
}
