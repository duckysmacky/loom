import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';

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

const DEFAULT_ACCENT: Record<NodeResponse['kind'], string> = {
	course: 'var(--node-course)',
	project: 'var(--node-blue)',
	idea: 'var(--node-idea)',
	path: 'var(--node-path)'
};

/** CSS color for a node's accent - its own color, else its kind's default. */
export function accentColor(node: NodeResponse): string {
	return node.color ?? DEFAULT_ACCENT[node.kind];
}

/** Focus-tier marker colors, shared by the Organized section bars and canvas cards. */
export const TIER_COLOR: Record<NodeResponse['focus'], string> = {
	primary: 'var(--accent)',
	secondary: 'var(--ink-2)',
	background: 'var(--line)'
};

/**
 * An unpromoted idea sitting in the backlog. These stay out of the board
 * views - they're waiting to be picked up, not part of the working graph.
 */
export function isBacklog(node: NodeResponse): boolean {
	return node.kind === 'idea' && node.status === 'idea';
}

/** Dashed border = not currently being worked on (queued, paused, or still an idea). */
export function isDashed(node: NodeResponse): boolean {
	return node.status === 'queued' || node.status === 'paused' || node.status === 'idea';
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

/** ISO timestamp → `yyyy-mm-dd` in local time, for `<input type="date">`. */
export function toDateInput(iso: string | null): string {
	if (!iso) return '';
	const date = new Date(iso);
	const pad = (value: number) => String(value).padStart(2, '0');
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** `yyyy-mm-dd` from a date input → ISO timestamp at local midnight; empty → null. */
export function fromDateInput(value: string): string | null {
	return value ? new Date(`${value}T00:00:00`).toISOString() : null;
}

/**
 * The progress a node shows, by precedence: its own tracked counter (e.g. a
 * course's videos), then its checklist, then its part_of children.
 */
export function progressOf(
	node: NodeResponse
): { done: number; total: number; unit: 'tracked' | 'tasks' | 'children' } | null {
	if (node.progress_current !== null && node.progress_total !== null) {
		return { done: node.progress_current, total: node.progress_total, unit: 'tracked' };
	}
	if (node.checklist_progress) return { ...node.checklist_progress, unit: 'tasks' };
	if (node.container_progress) return { ...node.container_progress, unit: 'children' };
	return null;
}

/** "15 of 30", "3 of 5 tasks", "1 of 2 done", or null. */
export function progressText(node: NodeResponse): string | null {
	const progress = progressOf(node);
	if (!progress) return null;
	const suffix = { tracked: '', tasks: ' tasks', children: ' done' }[progress.unit];
	return `${progress.done} of ${progress.total}${suffix}`;
}

/** [done, total] of whichever progress the node shows. */
export function progressPair(node: NodeResponse): [number, number] | null {
	const progress = progressOf(node);
	return progress ? [progress.done, progress.total] : null;
}

/** First non-heading line of the markdown notes, stripped of markup. */
export function notesExcerpt(notes: string | null): string {
	const line = (notes ?? '')
		.split('\n')
		.map((text) => text.trim())
		.find((text) => text && !text.startsWith('#'));
	return (line ?? '').replace(/[*_`>[\]]|^[-+] /g, '').trim();
}
