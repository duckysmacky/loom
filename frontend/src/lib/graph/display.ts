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
	'#5d6f85',
	'#b0483f'
] as const;

const DEFAULT_ACCENT: Record<NodeResponse['kind'], string> = {
	// The design system names this accent after the old "course" kind.
	study: 'var(--node-course)',
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
 * A backlog node: anything still at status `idea`, whatever its kind. These
 * stay out of the board views - captured, but not picked up yet.
 */
export function isBacklog(node: NodeResponse): boolean {
	return node.status === 'idea';
}

export type BorderStyle = {
	line: 'solid' | 'dashed' | 'dotted';
	tone: 'frame' | 'warn' | 'ok' | 'muted' | 'faded';
};

/**
 * Card border language. The line says what a node is (solid project/study,
 * dashed idea/path); only "paused" overrides it, with a dotted line. The
 * colour says how it's doing: warn while blocked, green once done, faded
 * while hidden (backlog/archived), muted grey while queued.
 */
export function borderStyle(node: NodeResponse): BorderStyle {
	const line =
		node.status === 'paused'
			? 'dotted'
			: node.kind === 'idea' || node.kind === 'path'
				? 'dashed'
				: 'solid';
	const tone =
		node.blocked && node.status !== 'done'
			? 'warn'
			: node.status === 'done'
				? 'ok'
				: node.status === 'archived' || node.status === 'idea'
					? 'faded'
					: node.status === 'queued'
						? 'muted'
						: 'frame';
	return { line, tone };
}

/** Typographic kind marks (the design system uses glyphs, never icons). */
export const KIND_GLYPH: Record<NodeResponse['kind'], string> = {
	project: '■',
	study: '◆',
	idea: '○',
	path: '▭'
};

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
 * The progress a node shows - each kind has exactly one source: a study's
 * tracked counter, a project's checklist, a path's contained nodes. Ideas
 * have none.
 */
export function progressOf(
	node: NodeResponse
): { done: number; total: number; unit: 'tracked' | 'tasks' | 'children' } | null {
	if (node.kind === 'study' && node.progress_current !== null && node.progress_total !== null) {
		return { done: node.progress_current, total: node.progress_total, unit: 'tracked' };
	}
	if (node.kind === 'project' && node.checklist_progress) {
		return { ...node.checklist_progress, unit: 'tasks' };
	}
	if (node.kind === 'path' && node.container_progress) {
		return { ...node.container_progress, unit: 'children' };
	}
	return null;
}

/**
 * What a node would lose by switching to `kind` - kinds are strict, so the
 * server drops data the new kind can't have. Empty when nothing is lost.
 */
export function kindChangeLosses(node: NodeResponse, kind: NodeResponse['kind']): string[] {
	const losses: string[] = [];
	if (kind !== 'study' && node.progress_total !== null) {
		losses.push(`the progress counter (${node.progress_current ?? 0} / ${node.progress_total})`);
	}
	if (kind !== 'project' && node.checklist_progress) {
		losses.push(`${node.checklist_progress.total} checklist item(s)`);
	}
	if (kind !== 'path' && node.container_progress) {
		losses.push(`${node.container_progress.total} contained node(s), which leave the path`);
	}
	return losses;
}

/** "15 / 30 videos" (unit label optional), "3 / 5 tasks", "1 / 2 done", or null. */
export function progressText(node: NodeResponse): string | null {
	const progress = progressOf(node);
	if (!progress) return null;
	const suffix = {
		tracked: node.progress_unit ? ` ${node.progress_unit}` : '',
		tasks: ' tasks',
		children: ' done'
	}[progress.unit];
	return `${progress.done} / ${progress.total}${suffix}`;
}

/** The tracked counter after a quick "+1", or null when there's nothing to bump. */
export function incrementedProgress(node: NodeResponse): number | null {
	const { progress_current: current, progress_total: total } = node;
	if (current === null || total === null || current >= total) return null;
	return current + 1;
}

/** [done, total] of whichever progress the node shows. */
export function progressPair(node: NodeResponse): [number, number] | null {
	const progress = progressOf(node);
	return progress ? [progress.done, progress.total] : null;
}

/** The first `lines` prose lines of the markdown notes (headings skipped), stripped of markup. */
export function notesExcerpt(notes: string | null, lines = 1): string {
	return (notes ?? '')
		.split('\n')
		.map((text) => text.trim())
		.filter((text) => text && !text.startsWith('#'))
		.slice(0, lines)
		.map((text) => text.replace(/[*_`>[\]]|^[-+] /g, '').trim())
		.join(' ');
}
