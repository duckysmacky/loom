import { describe, expect, it } from 'vitest';
import { makeNode, makePeriod } from './fixtures';
import {
	DAY_MS,
	barMarks,
	barSpan,
	clampZoom,
	ghostSpan,
	nodeBarSpans,
	periodSpans,
	pokeOffsets,
	scrollAfterZoom,
	startOfDay,
	startOfWeek,
	timelineEligible,
	timelineRange,
	weekStarts,
	xOf
} from './timeline';

// A Wednesday.
const now = new Date('2026-09-23T12:00:00').getTime();

// Day-snapped spans: 10 px per day from 1 Sept (local time).
const local = (iso: string) => new Date(iso).getTime();
const range = { start: local('2026-09-01T00:00:00'), end: local('2026-12-01T00:00:00') };
const PX = 10;
const iso = (local_time: string) => new Date(local_time).toISOString();

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

	it('snaps to local midnight', () => {
		expect(startOfDay(now)).toBe(local('2026-09-23T00:00:00'));
	});

	it('runs open bars to the end of today and closed bars to the end of their last day', () => {
		const started = iso('2026-09-10T15:30:00');
		expect(barSpan(makeNode({ started_at: null }), range, PX, now)).toBeNull();
		expect(barSpan(makeNode({ started_at: started }), range, PX, now)).toEqual({
			left: 90,
			width: 140,
			focus: 'secondary',
			kind: 'period'
		});
		const closed = makeNode({ started_at: started, completed_at: iso('2026-09-12T08:00:00') });
		expect(barSpan(closed, range, PX, now)).toMatchObject({ left: 90, width: 30 });
	});

	it('draws a same-day period as one full day, whatever the time', () => {
		const sameDay = makeNode({
			started_at: iso('2026-09-10T09:00:00'),
			completed_at: iso('2026-09-10T09:05:00')
		});
		expect(barSpan(sameDay, range, PX, now)).toMatchObject({ left: 90, width: 10 });
		// Started today: still one full day, no sliver.
		expect(
			barSpan(makeNode({ started_at: iso('2026-09-23T11:00:00') }), range, PX, now)
		).toMatchObject({ left: 220, width: 10 });
	});

	it('places every poke tick that falls inside the range, in order', () => {
		const range = { start: 0, end: 100 * DAY_MS };
		const inside = new Date(25 * DAY_MS).toISOString();
		const alsoInside = new Date(60 * DAY_MS).toISOString();
		const outside = new Date(250 * DAY_MS).toISOString();
		expect(pokeOffsets([inside, outside, alsoInside], range, 2)).toEqual([50, 120]);
	});

	it('lays out one span per active period, open ones running to today', () => {
		const periods = [
			makePeriod({
				started_at: iso('2026-09-10T10:00:00'),
				ended_at: iso('2026-09-19T20:00:00')
			}),
			makePeriod({ started_at: iso('2026-09-21T10:00:00'), ended_at: null, focus: 'primary' })
		];
		expect(periodSpans(periods, range, PX, now)).toEqual([
			{ left: 90, width: 100, focus: 'secondary', kind: 'period' },
			{ left: 200, width: 30, focus: 'primary', kind: 'period' }
		]);
	});

	it('does not stack periods that share a day', () => {
		const sameDay = [
			makePeriod({
				started_at: iso('2026-09-10T09:00:00'),
				ended_at: iso('2026-09-10T11:00:00')
			}),
			makePeriod({
				started_at: iso('2026-09-10T14:00:00'),
				ended_at: iso('2026-09-10T18:00:00'),
				focus: 'primary'
			})
		];
		// The later one leaves no days of its own: one day, in the later focus.
		expect(periodSpans(sameDay, range, PX, now)).toEqual([
			{ left: 90, width: 10, focus: 'primary', kind: 'period' }
		]);
		const overlapping = [
			makePeriod({
				started_at: iso('2026-09-10T09:00:00'),
				ended_at: iso('2026-09-12T11:00:00')
			}),
			makePeriod({
				started_at: iso('2026-09-12T14:00:00'),
				ended_at: iso('2026-09-14T18:00:00'),
				focus: 'primary'
			})
		];
		expect(periodSpans(overlapping, range, PX, now)).toEqual([
			{ left: 90, width: 30, focus: 'secondary', kind: 'period' },
			{ left: 120, width: 20, focus: 'primary', kind: 'period' }
		]);
	});

	it('prefers periods when present, else falls back to the classic bar', () => {
		const node = makeNode({ started_at: iso('2026-09-10T10:00:00') });
		const period = makePeriod({ started_at: iso('2026-09-20T10:00:00'), ended_at: null });

		expect(nodeBarSpans(node, [period], range, PX, now)).toEqual(
			periodSpans([period], range, PX, now)
		);
		expect(nodeBarSpans(node, [], range, PX, now)).toEqual([barSpan(node, range, PX, now)]);
		expect(nodeBarSpans(makeNode({ started_at: null }), [], range, PX, now)).toEqual([]);
	});

	describe('paused ghost', () => {
		const paused = makeNode({ status: 'paused', started_at: iso('2026-09-10T10:00:00') });
		const closed = makePeriod({
			started_at: iso('2026-09-10T10:00:00'),
			ended_at: iso('2026-09-15T10:00:00')
		});

		it('fills the days since the last period ended, through today', () => {
			expect(ghostSpan(paused, [closed], range, PX, now)).toEqual({
				left: 150,
				width: 80,
				focus: null,
				kind: 'ghost'
			});
		});

		it('is added only when paused nodes are shown', () => {
			expect(nodeBarSpans(paused, [closed], range, PX, now)).toHaveLength(1);
			expect(nodeBarSpans(paused, [closed], range, PX, now, true).map((span) => span.kind)).toEqual(
				['period', 'ghost']
			);
		});

		it('never appears for done nodes, open periods, today or periodless nodes', () => {
			expect(ghostSpan({ ...paused, status: 'done' }, [closed], range, PX, now)).toBeNull();
			expect(ghostSpan(paused, [{ ...closed, ended_at: null }], range, PX, now)).toBeNull();
			const today = { ...closed, ended_at: iso('2026-09-23T08:00:00') };
			expect(ghostSpan(paused, [today], range, PX, now)).toBeNull();
			expect(ghostSpan(paused, [], range, PX, now)).toBeNull();
		});
	});

	it('zooms around the pointer and clamps the zoom level', () => {
		// Pointer over day 30 (scrolled 200px + 100px in, at 10px/day); at 20px/day
		// day 30 sits at 600px, so it stays under the pointer at scroll 500.
		expect(scrollAfterZoom(200, 100, 10, 20)).toBe(500);
		expect(clampZoom(1000)).toBe(60);
		expect(clampZoom(0.1)).toBe(2);
	});
});

describe('timelineEligible', () => {
	it('keeps only projects and studies that are active or done', () => {
		expect(timelineEligible(makeNode({ kind: 'project', status: 'active' }), false)).toBe(true);
		expect(timelineEligible(makeNode({ kind: 'study', status: 'done' }), false)).toBe(true);
		for (const status of ['queued', 'idea', null] as const) {
			expect(timelineEligible(makeNode({ status }), true)).toBe(false);
		}
		expect(timelineEligible(makeNode({ kind: 'path' }), true)).toBe(false);
		expect(timelineEligible(makeNode({ kind: 'idea', status: null }), true)).toBe(false);
	});

	it('shows paused nodes only when asked', () => {
		const paused = makeNode({ status: 'paused' });
		expect(timelineEligible(paused, false)).toBe(false);
		expect(timelineEligible(paused, true)).toBe(true);
	});
});

describe('barMarks', () => {
	const period = { left: 0, width: 10, focus: 'secondary', kind: 'period' } as const;
	const ghost = { ...period, kind: 'ghost' } as const;

	it('marks the latest bar done, or blocked, with done winning', () => {
		expect(barMarks(makeNode({ status: 'done' }), period, true)).toEqual({
			done: true,
			blocked: false,
			ghost: false
		});
		expect(barMarks(makeNode({ blocked: true }), period, true).blocked).toBe(true);
		expect(barMarks(makeNode({ status: 'done', blocked: true }), period, true)).toMatchObject({
			done: true,
			blocked: false
		});
	});

	it('leaves earlier bars and ghosts unmarked', () => {
		expect(barMarks(makeNode({ status: 'done' }), period, false).done).toBe(false);
		expect(barMarks(makeNode({ blocked: true }), period, false).blocked).toBe(false);
		expect(barMarks(makeNode({ status: 'paused', blocked: true }), ghost, true)).toEqual({
			done: false,
			blocked: false,
			ghost: true
		});
	});
});
