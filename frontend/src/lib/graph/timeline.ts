import type { NodeResponse } from '$lib/types/NodeResponse';
import { shortDate } from './display';

const DAY_MS = 86_400_000;
const WEEK_MS = 7 * DAY_MS;
const MIN_WEEKS = 4;

/** Monday 00:00 (local time) of the week containing `time`. */
export function startOfWeek(time: number): number {
	const date = new Date(time);
	date.setHours(0, 0, 0, 0);
	date.setDate(date.getDate() - ((date.getDay() + 6) % 7));
	return date.getTime();
}

export type TimelineWindow = {
	start: number;
	end: number;
	weeks: { start: number; label: string }[];
};

/**
 * Week columns from the week of the earliest start to the current week
 * (inclusive), at least `MIN_WEEKS` wide.
 */
export function timelineWindow(nodes: NodeResponse[], now = Date.now()): TimelineWindow {
	const starts = nodes.flatMap((node) => (node.started_at ? [Date.parse(node.started_at)] : []));
	const end = startOfWeek(now) + WEEK_MS;
	const earliest = startOfWeek(Math.min(now, ...starts));
	const start = Math.min(earliest, end - MIN_WEEKS * WEEK_MS);

	const weeks = [];
	for (let weekStart = start; weekStart < end; weekStart += WEEK_MS) {
		weeks.push({ start: weekStart, label: shortDate(new Date(weekStart).toISOString()) });
	}
	return { start, end, weeks };
}

const percentOf = (time: number, window: TimelineWindow) =>
	((time - window.start) / (window.end - window.start)) * 100;

/**
 * Horizontal span of a node's bar, in percent of the window: started_at to
 * completed_at, or to now while still open. `null` for never-started nodes.
 */
export function barSpan(
	node: NodeResponse,
	window: TimelineWindow,
	now = Date.now()
): { left: number; width: number } | null {
	if (!node.started_at) return null;
	const start = Date.parse(node.started_at);
	const finish = node.completed_at ? Date.parse(node.completed_at) : now;
	const left = Math.max(0, percentOf(start, window));
	const right = Math.min(100, percentOf(Math.max(finish, start), window));
	// At least a sliver, so a node started today is still visible.
	return { left, width: Math.max(right - left, 0.8) };
}

/** Position of the node's last poke, in percent of the window. */
export function pokeOffset(node: NodeResponse, window: TimelineWindow): number | null {
	if (!node.last_poked_at) return null;
	const offset = percentOf(Date.parse(node.last_poked_at), window);
	return offset >= 0 && offset <= 100 ? offset : null;
}
