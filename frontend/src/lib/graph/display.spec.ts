import { describe, expect, it } from 'vitest';
import {
	accentColor,
	childrenOf,
	isBacklog,
	isDashed,
	notesExcerpt,
	progressText,
	relativeDays,
	requirementsOf,
	shortDate
} from './display';
import { makeEdge, makeNode } from './fixtures';

describe('display helpers', () => {
	it('treats only unpromoted ideas as backlog', () => {
		expect(isBacklog(makeNode({ kind: 'idea', status: 'idea' }))).toBe(true);
		expect(isBacklog(makeNode({ kind: 'idea', status: 'queued' }))).toBe(false);
		expect(isBacklog(makeNode({ kind: 'project', status: 'idea' }))).toBe(false);
	});

	it('dashes nodes that are not being worked on', () => {
		expect(isDashed(makeNode({ status: 'queued' }))).toBe(true);
		expect(isDashed(makeNode({ status: 'paused' }))).toBe(true);
		expect(isDashed(makeNode({ status: 'active' }))).toBe(false);
		expect(isDashed(makeNode({ status: 'done' }))).toBe(false);
	});

	it('uses the node color, falling back to the kind default', () => {
		expect(accentColor(makeNode({ color: '#123456' }))).toBe('#123456');
		expect(accentColor(makeNode({ kind: 'course' }))).toBe('var(--node-course)');
		expect(accentColor(makeNode({ kind: 'path' }))).toBe('var(--node-path)');
	});

	it('flags requirements met only when the target is done', () => {
		const renderer = makeNode({ id: 'renderer' });
		const course = makeNode({ id: 'course', status: 'active' });
		const math = makeNode({ id: 'math', status: 'done' });
		const byId = new Map([renderer, course, math].map((node) => [node.id, node]));
		const edges = [
			makeEdge('renderer', 'course', 'requires'),
			makeEdge('renderer', 'math', 'requires'),
			makeEdge('renderer', 'math', 'related')
		];

		expect(requirementsOf('renderer', edges, byId).map(({ node, met }) => [node.id, met])).toEqual([
			['course', false],
			['math', true]
		]);
	});

	it('lists part_of children of a container', () => {
		const path = makeNode({ id: 'path' });
		const child = makeNode({ id: 'child' });
		const byId = new Map([path, child].map((node) => [node.id, node]));
		expect(childrenOf('path', [makeEdge('child', 'path', 'part_of')], byId)).toEqual([child]);
	});

	it('formats progress for tracked counts and containers', () => {
		expect(progressText(makeNode({ progress_current: 15, progress_total: 30 }))).toBe('15 of 30');
		expect(progressText(makeNode({ container_progress: { done: 1, total: 2 } }))).toBe(
			'1 of 2 done'
		);
		expect(progressText(makeNode())).toBeNull();
	});

	it('formats relative days', () => {
		const now = Date.parse('2026-09-10T12:00:00Z');
		expect(relativeDays('2026-09-10T08:00:00Z', now)).toBe('today');
		expect(relativeDays('2026-09-05T12:00:00Z', now)).toBe('5d ago');
	});

	it('formats short dates, adding the year outside the current one', () => {
		const now = new Date('2026-09-10T12:00:00');
		expect(shortDate('2026-09-02T12:00:00', now)).toBe('2 Sep');
		expect(shortDate('2025-07-02T12:00:00', now)).toBe('2 Jul 2025');
	});

	it('takes the first prose line of the notes as the excerpt', () => {
		expect(notesExcerpt('# Plan\n\nA **from-scratch** renderer.\nMore')).toBe(
			'A from-scratch renderer.'
		);
		expect(notesExcerpt(null)).toBe('');
	});
});
