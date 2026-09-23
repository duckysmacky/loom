import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';

/** Kind as shown on badges - a `part_of` container reads as a "path". */
export type DisplayKind = NodeResponse['kind'] | 'path';

export function displayKind(node: NodeResponse): DisplayKind {
	return node.container_progress ? 'path' : node.kind;
}

/**
 * Fixed node-accent palette from the design system (identity, never status).
 * Stored on nodes/topics as the hex value - the API only accepts hex colors.
 */
export const ACCENT_PALETTE = [
	'#3f6f8f',
	'#2f6f6a',
	'#7a5c8f',
	'#c26b2c',
	'#b5893a',
	'#c9a13c',
	'#8a7fa3',
	'#6b7b62',
	'#5d6f85'
] as const;

const DEFAULT_ACCENT: Record<DisplayKind, string> = {
	course: 'var(--node-course)',
	project: 'var(--node-blue)',
	idea: 'var(--node-idea)',
	path: 'var(--node-path)'
};

/** CSS color for a node's accent - its own color, else its kind's default. */
export function accentColor(node: NodeResponse): string {
	return node.color ?? DEFAULT_ACCENT[displayKind(node)];
}

export type Requirement = { node: NodeResponse; met: boolean };

/** Nodes this one `requires`, each flagged met (`done`) or unmet. */
export function requirementsOf(
	nodeId: string,
	edges: EdgeResponse[],
	nodeById: Map<string, NodeResponse>
): Requirement[] {
	return edges
		.filter((edge) => edge.kind === 'requires' && edge.from_node_id === nodeId)
		.flatMap((edge) => {
			const target = nodeById.get(edge.to_node_id);
			return target ? [{ node: target, met: target.status === 'done' }] : [];
		});
}

/** `part_of` children of a container, in edge order. */
export function childrenOf(
	nodeId: string,
	edges: EdgeResponse[],
	nodeById: Map<string, NodeResponse>
): NodeResponse[] {
	return edges
		.filter((edge) => edge.kind === 'part_of' && edge.to_node_id === nodeId)
		.flatMap((edge) => nodeById.get(edge.from_node_id) ?? []);
}

const DAY_MS = 86_400_000;

export function daysSince(iso: string, now = Date.now()): number {
	return Math.max(0, Math.floor((now - Date.parse(iso)) / DAY_MS));
}

/** "today", "1d ago", "34d ago". */
export function relativeDays(iso: string, now = Date.now()): string {
	const days = daysSince(iso, now);
	return days === 0 ? 'today' : `${days}d ago`;
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** "2 Sep", or "2 Jul 2025" outside the current year. */
export function shortDate(iso: string, now = new Date()): string {
	const date = new Date(iso);
	const dayMonth = `${date.getDate()} ${MONTHS[date.getMonth()]}`;
	return date.getFullYear() === now.getFullYear() ? dayMonth : `${dayMonth} ${date.getFullYear()}`;
}

/** "15 of 30" for tracked progress, "1 of 2 done" for containers, else null. */
export function progressText(node: NodeResponse): string | null {
	if (node.progress_current !== null && node.progress_total !== null) {
		return `${node.progress_current} of ${node.progress_total}`;
	}
	if (node.container_progress) {
		return `${node.container_progress.done} of ${node.container_progress.total} done`;
	}
	return null;
}

/** [current, total] for whichever progress the node carries. */
export function progressPair(node: NodeResponse): [number, number] | null {
	if (node.progress_current !== null && node.progress_total !== null) {
		return [node.progress_current, node.progress_total];
	}
	if (node.container_progress) {
		return [node.container_progress.done, node.container_progress.total];
	}
	return null;
}

/** First non-heading line of the markdown notes, stripped of markup. */
export function notesExcerpt(notes: string | null): string {
	const line = (notes ?? '')
		.split('\n')
		.map((text) => text.trim())
		.find((text) => text && !text.startsWith('#'));
	return (line ?? '').replace(/[*_`>[\]]|^[-+] /g, '').trim();
}
