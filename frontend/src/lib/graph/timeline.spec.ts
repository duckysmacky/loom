import { describe, expect, it } from 'vitest';
import { makeNode } from './fixtures';
import { barSpan, pokeOffset, startOfWeek, timelineWindow } from './timeline';

// A Wednesday.
const now = new Date('2026-09-23T12:00:00').getTime();

describe('timeline math', () => {
	it('snaps to Monday midnight', () => {
		expect(startOfWeek(now)).toBe(new Date('2026-09-21T00:00:00').getTime());
	});

	it('spans from the earliest start week to the current week, at least 4 weeks', () => {
		expect(timelineWindow([], now).weeks).toHaveLength(4);

		const early = makeNode({ started_at: new Date('2026-08-05T10:00:00').toISOString() });
		const window = timelineWindow([early], now);
		expect(window.weeks[0].start).toBe(startOfWeek(Date.parse(early.started_at!)));
		expect(window.weeks.at(-1)!.start).toBe(startOfWeek(now));
	});

	it('runs open bars to now and closed bars to completion', () => {
		const window = { start: 0, end: 100, weeks: [] };
		expect(barSpan(makeNode({ started_at: null }), window, 50)).toBeNull();
		expect(barSpan(makeNode({ started_at: new Date(10).toISOString() }), window, 60)).toEqual({
			left: 10,
			width: 50
		});
		expect(
			barSpan(
				makeNode({
					started_at: new Date(10).toISOString(),
					completed_at: new Date(30).toISOString()
				}),
				window,
				60
			)
		).toEqual({ left: 10, width: 20 });
	});

	it('places the poke tick only inside the window', () => {
		const window = { start: 0, end: 100, weeks: [] };
		expect(pokeOffset(makeNode({ last_poked_at: new Date(25).toISOString() }), window)).toBe(25);
		expect(pokeOffset(makeNode({ last_poked_at: new Date(250).toISOString() }), window)).toBeNull();
	});
});
