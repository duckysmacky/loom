import type { EdgeKind } from '$lib/types/EdgeKind';
import type { EdgeStyle } from '$lib/types/EdgeStyle';
import type { EdgeStyles } from '$lib/types/EdgeStyles';
import type { LineWidth } from '$lib/types/LineWidth';

export type EdgeStyleKey = keyof EdgeStyles;

const style = (line: EdgeStyle['line'], width: LineWidth, arrow: boolean): EdgeStyle => ({
	color: null,
	line,
	width,
	arrow
});

/** Mirrors `EdgeStyles::default()` in the backend; a null colour = the theme token. */
export const DEFAULT_EDGE_STYLES: EdgeStyles = {
	related: style('dashed', 'thin', false),
	precedes: style('dashed', 'thin', true),
	requires_unmet: style('solid', 'normal', true),
	requires_met: style('solid', 'normal', true)
};

/** Theme token a style falls back to while it has no colour of its own. */
export const EDGE_DEFAULT_COLOR: Record<EdgeStyleKey, string> = {
	related: 'var(--ink-2)',
	precedes: 'var(--edge-precedes)',
	requires_unmet: 'var(--warn)',
	requires_met: 'var(--ok)'
};

/** Which style a connection uses; `part_of` is drawn as the path box and has none. */
export function edgeStyleKey(kind: EdgeKind, unmet: boolean): EdgeStyleKey | null {
	if (kind === 'part_of') return null;
	if (kind === 'requires') return unmet ? 'requires_unmet' : 'requires_met';
	return kind;
}

const WIDTHS: Record<LineWidth, number> = { thin: 1.5, normal: 2, thick: 3 };

/** SVG stroke attributes for a style (colour stays null so the CSS token default applies). */
export function edgeStrokeProps(edgeStyle: EdgeStyle) {
	return {
		stroke: edgeStyle.color,
		width: WIDTHS[edgeStyle.width],
		dasharray: { solid: 'none', dashed: '7 5', dotted: '1 5' }[edgeStyle.line],
		linecap: edgeStyle.line === 'dotted' ? 'round' : 'butt'
	};
}

export type HandleSide = 'left' | 'right' | 'top' | 'bottom';

const ARROW_LENGTH = 9;
const ARROW_HALF_WIDTH = 5;

/** Triangle with its tip on the target handle, pointing into the node it enters. */
export function arrowPath(x: number, y: number, side: HandleSide): string {
	const alongX = side === 'left' ? -1 : side === 'right' ? 1 : 0;
	const alongY = side === 'top' ? -1 : side === 'bottom' ? 1 : 0;
	const baseX = x + alongX * ARROW_LENGTH;
	const baseY = y + alongY * ARROW_LENGTH;
	// Perpendicular to the approach direction.
	const spreadX = alongY * ARROW_HALF_WIDTH;
	const spreadY = alongX * ARROW_HALF_WIDTH;
	return `M ${x} ${y} L ${baseX + spreadX} ${baseY + spreadY} L ${baseX - spreadX} ${baseY - spreadY} Z`;
}
