import type { ActivePeriodResponse } from '$lib/types/ActivePeriodResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';

export const DAY_MS = 86_400_000;
const WEEK_MS = 7 * DAY_MS;
/** Weeks of runway before the earliest start and after today. */
const LEAD_WEEKS = 1;
const TRAIL_WEEKS = 8;

export const MIN_PX_PER_DAY = 2;
export const MAX_PX_PER_DAY = 60;
export const DEFAULT_PX_PER_DAY = 12;

/** Monday 00:00 (local time) of the week containing `time`. */
export function startOfWeek(time: number): number {
	const date = new Date(time);
	date.setHours(0, 0, 0, 0);
	date.setDate(date.getDate() - ((date.getDay() + 6) % 7));
	return date.getTime();
}

/** Time span the timeline covers: from a week before the earliest start to 8 weeks past today. */
export type TimelineRange = { start: number; end: number };

export function timelineRange(nodes: NodeResponse[], now = Date.now()): TimelineRange {
	const starts = nodes.flatMap((node) => (node.started_at ? [Date.parse(node.started_at)] : []));
	return {
		start: startOfWeek(Math.min(now, ...starts)) - LEAD_WEEKS * WEEK_MS,
		end: startOfWeek(now) + (TRAIL_WEEKS + 1) * WEEK_MS
	};
}

/** Horizontal pixel offset of `time` within the range. */
export function xOf(time: number, range: TimelineRange, pxPerDay: number): number {
	return ((time - range.start) / DAY_MS) * pxPerDay;
}

export function rangeWidth(range: TimelineRange, pxPerDay: number): number {
	return xOf(range.end, range, pxPerDay);
}

/** Monday timestamps of every week in the range. */
export function weekStarts(range: TimelineRange): number[] {
	const weeks = [];
	for (let week = startOfWeek(range.start); week < range.end; week += WEEK_MS) weeks.push(week);
	return weeks;
}

/**
 * A node's bar in pixels: started_at to completed_at, or to now while still
 * open. `null` for never-started nodes. At least a sliver wide.
 */
export function barSpan(
	node: NodeResponse,
	range: TimelineRange,
	pxPerDay: number,
	now = Date.now()
): Span | null {
	if (!node.started_at) return null;
	const start = Date.parse(node.started_at);
	const finish = node.completed_at ? Date.parse(node.completed_at) : now;
	const left = xOf(start, range, pxPerDay);
	return { left, width: Math.max(xOf(Math.max(finish, start), range, pxPerDay) - left, 4) };
}

export type Span = { left: number; width: number };

/** One span per active period - an open one (`ended_at` null) runs to `now`. */
export function periodSpans(
	periods: ActivePeriodResponse[],
	range: TimelineRange,
	pxPerDay: number,
	now = Date.now()
): Span[] {
	return periods.map((period) => {
		const start = Date.parse(period.started_at);
		const finish = period.ended_at ? Date.parse(period.ended_at) : now;
		const left = xOf(start, range, pxPerDay);
		return { left, width: Math.max(xOf(Math.max(finish, start), range, pxPerDay) - left, 4) };
	});
}

/**
 * The spans to draw for one node: its active periods when it has any (the
 * fine-grained truth), else the classic single started_at->completed_at
 * bar - which itself is `[]` for a never-started node.
 */
export function nodeBarSpans(
	node: NodeResponse,
	periods: ActivePeriodResponse[],
	range: TimelineRange,
	pxPerDay: number,
	now = Date.now()
): Span[] {
	if (periods.length) return periodSpans(periods, range, pxPerDay, now);
	const classic = barSpan(node, range, pxPerDay, now);
	return classic ? [classic] : [];
}

/** Pixel offset of every in-range poke, not just the latest. */
export function pokeOffsets(pokedAts: string[], range: TimelineRange, pxPerDay: number): number[] {
	return pokedAts
		.map((iso) => Date.parse(iso))
		.filter((time) => time >= range.start && time <= range.end)
		.map((time) => xOf(time, range, pxPerDay));
}

export function clampZoom(pxPerDay: number): number {
	return Math.min(MAX_PX_PER_DAY, Math.max(MIN_PX_PER_DAY, pxPerDay));
}

/**
 * New scroll offset after zooming, keeping the date under the pointer in
 * place. `pointerX` is measured from the start of the time axis in the
 * viewport (i.e. after the fixed name column).
 */
export function scrollAfterZoom(
	scrollLeft: number,
	pointerX: number,
	fromPxPerDay: number,
	toPxPerDay: number
): number {
	return ((scrollLeft + pointerX) / fromPxPerDay) * toPxPerDay - pointerX;
}
