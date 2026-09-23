import { describe, expect, it } from 'vitest';
import { makeNode } from './fixtures';
import {
	DAY_MS,
	barSpan,
	clampZoom,
	pokeOffset,
	scrollAfterZoom,
	startOfWeek,
	timelineRange,
	weekStarts,
	xOf
} from './timeline';

// A Wednesday.
const now = new Date('2026-09-23T12:00:00').getTime();

describe('timeline math', () => {
	it('snaps to Monday midnight', () => {
		expect(startOfWeek(now)).toBe(new Date('2026-09-21T00:00:00').getTime());
	});

	it('runs from a week before the earliest start to eight weeks past today', () => {
		const early = makeNode({ started_at: new Date('2026-08-05T10:00:00').toISOString() });
		const range = timelineRange([early], now);
		expect(range.start).toBe(startOfWeek(Date.parse(early.started_at!)) - 7 * DAY_MS);
		expect(range.end).toBe(startOfWeek(now) + 9 * 7 * DAY_MS);
		expect(weekStarts(range)[0]).toBe(range.start);
	});

	it('maps time to pixels at the current zoom', () => {
		const range = { start: 0, end: 10 * DAY_MS };
		expect(xOf(3 * DAY_MS, range, 12)).toBe(36);
		expect(xOf(3 * DAY_MS, range, 24)).toBe(72);
	});

	it('runs open bars to now and closed bars to completion, in pixels', () => {
		const range = { start: 0, end: 100 * DAY_MS };
		const started = new Date(10 * DAY_MS).toISOString();
		expect(barSpan(makeNode({ started_at: null }), range, 10, 50 * DAY_MS)).toBeNull();
		expect(barSpan(makeNode({ started_at: started }), range, 10, 60 * DAY_MS)).toEqual({
			left: 100,
			width: 500
		});
		expect(
			barSpan(
				makeNode({ started_at: started, completed_at: new Date(30 * DAY_MS).toISOString() }),
				range,
				10,
				60 * DAY_MS
			)
		).toEqual({ left: 100, width: 200 });
		// A node started today is still visible as a sliver.
		expect(barSpan(makeNode({ started_at: started }), range, 10, 10 * DAY_MS)!.width).toBe(4);
	});

	it('places the poke tick only inside the range', () => {
		const range = { start: 0, end: 100 * DAY_MS };
		const inside = new Date(25 * DAY_MS).toISOString();
		const outside = new Date(250 * DAY_MS).toISOString();
		expect(pokeOffset(makeNode({ last_poked_at: inside }), range, 2)).toBe(50);
		expect(pokeOffset(makeNode({ last_poked_at: outside }), range, 2)).toBeNull();
	});

	it('zooms around the pointer and clamps the zoom level', () => {
		// Pointer over day 30 (scrolled 200px + 100px in, at 10px/day); at 20px/day
		// day 30 sits at 600px, so it stays under the pointer at scroll 500.
		expect(scrollAfterZoom(200, 100, 10, 20)).toBe(500);
		expect(clampZoom(1000)).toBe(60);
		expect(clampZoom(0.1)).toBe(2);
	});
});
