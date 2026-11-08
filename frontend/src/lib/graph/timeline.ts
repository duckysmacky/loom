import type { ActivePeriodResponse } from '$lib/types/ActivePeriodResponse';
import type { NodeFocus } from '$lib/types/NodeFocus';
import type { NodeResponse } from '$lib/types/NodeResponse';
import type { TimelineStyles } from '$lib/types/TimelineStyles';

/** Mirrors `TimelineStyles::default()` in the backend; a null colour = the theme token. */
export const DEFAULT_TIMELINE_STYLES: TimelineStyles = {
	primary: { color: null, fill: 'solid' },
	secondary: { color: null, fill: 'solid' },
	background: { color: null, fill: 'striped' },
	done_color: null,
	blocked_color: null,
	show_paused: false
};

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

/** Local midnight of the day containing `time`. */
export function startOfDay(time: number): number {
	const date = new Date(time);
	date.setHours(0, 0, 0, 0);
	return date.getTime();
}

/** Local midnight of the day after the one containing `time` (DST-safe, unlike `+ DAY_MS`). */
function startOfNextDay(time: number): number {
	const date = new Date(time);
	date.setHours(0, 0, 0, 0);
	date.setDate(date.getDate() + 1);
	return date.getTime();
}

/** `period` is real activity; `ghost` is the stretch a paused node has sat idle since. */
export type SpanKind = 'period' | 'ghost';

/** `focus` is the tier the node had during the span - what the timeline styles it by. */
export type Span = { left: number; width: number; focus: NodeFocus | null; kind: SpanKind };

/** A span in whole days, as local midnights: `start` inclusive, `end` the midnight after the last day. */
type DayRun = { start: number; end: number; focus: NodeFocus | null; kind: SpanKind };

/** Only days matter: any time within a day fills the whole day, so a same-day period is one day wide. */
function dayRun(
	start: number,
	end: number,
	focus: NodeFocus | null,
	kind: SpanKind = 'period'
): DayRun {
	return { start: startOfDay(start), end: startOfNextDay(Math.max(start, end)), focus, kind };
}

function toSpan(run: DayRun, range: TimelineRange, pxPerDay: number): Span {
	const left = xOf(run.start, range, pxPerDay);
	return { left, width: xOf(run.end, range, pxPerDay) - left, focus: run.focus, kind: run.kind };
}

/**
 * A node's bar: started_at to completed_at, or to today while still open.
 * `null` for never-started nodes.
 */
export function barSpan(
	node: NodeResponse,
	range: TimelineRange,
	pxPerDay: number,
	now = Date.now()
): Span | null {
	if (!node.started_at) return null;
	const finish = node.completed_at ? Date.parse(node.completed_at) : now;
	return toSpan(dayRun(Date.parse(node.started_at), finish, node.focus ?? null), range, pxPerDay);
}

/**
 * One span per active period - an open one (`ended_at` null) runs to today.
 * Periods sharing a day (paused and resumed within it) don't stack: the later
 * one only takes the days the earlier one left, or just its focus if it
 * leaves none.
 */
export function periodSpans(
	periods: ActivePeriodResponse[],
	range: TimelineRange,
	pxPerDay: number,
	now = Date.now()
): Span[] {
	const runs: DayRun[] = [];
	for (const period of periods) {
		const run = dayRun(
			Date.parse(period.started_at),
			period.ended_at ? Date.parse(period.ended_at) : now,
			period.focus
		);
		const previous = runs.at(-1);
		if (previous && run.start < previous.end) {
			if (run.end <= previous.end) {
				previous.focus = run.focus;
				continue;
			}
			run.start = previous.end;
		}
		runs.push(run);
	}
	return runs.map((run) => toSpan(run, range, pxPerDay));
}

/**
 * Where a paused node's last period ended up to today, as one ghost span.
 * `null` when it isn't paused, has no closed last period (never started, or
 * tracking left it open) or was paused today.
 */
export function ghostSpan(
	node: NodeResponse,
	periods: ActivePeriodResponse[],
	range: TimelineRange,
	pxPerDay: number,
	now = Date.now()
): Span | null {
	const last = periods.at(-1);
	if (node.status !== 'paused' || !last?.ended_at) return null;
	const start = startOfNextDay(Date.parse(last.ended_at));
	const end = startOfNextDay(now);
	if (start >= end) return null;
	return toSpan({ start, end, focus: null, kind: 'ghost' }, range, pxPerDay);
}

/**
 * The spans to draw for one node: its active periods when it has any (the
 * fine-grained truth), else the classic single started_at->completed_at
 * bar - which itself is `[]` for a never-started node. With `showPaused`, a
 * paused node also gets its ghost span.
 */
export function nodeBarSpans(
	node: NodeResponse,
	periods: ActivePeriodResponse[],
	range: TimelineRange,
	pxPerDay: number,
	now = Date.now(),
	showPaused = false
): Span[] {
	if (!periods.length) {
		const classic = barSpan(node, range, pxPerDay, now);
		return classic ? [classic] : [];
	}
	const spans = periodSpans(periods, range, pxPerDay, now);
	const ghost = showPaused ? ghostSpan(node, periods, range, pxPerDay, now) : null;
	return ghost ? [...spans, ghost] : spans;
}

/** Only projects and studies get rows, and only the statuses worth plotting (paused is opt-in). */
export function timelineEligible(node: NodeResponse, showPaused: boolean): boolean {
	if (node.kind !== 'project' && node.kind !== 'study') return false;
	return (
		node.status === 'active' || node.status === 'done' || (showPaused && node.status === 'paused')
	);
}

export type BarMarks = { done: boolean; blocked: boolean; ghost: boolean };

/**
 * Status marks drawn as borders over the focus fill. They describe where the
 * node is now, so only its latest bar carries them; done wins over blocked.
 */
export function barMarks(node: NodeResponse, span: Span, isLatest: boolean): BarMarks {
	const current = isLatest && span.kind === 'period';
	const done = current && node.status === 'done';
	return { done, blocked: current && !done && node.blocked, ghost: span.kind === 'ghost' };
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
