import { describe, expect, it } from 'vitest';
import { makeEdge, makeNode } from './fixtures';
import { parentPathOf } from './paths';
import {
	CANVAS_NODE_HEIGHT,
	CANVAS_NODE_WIDTH,
	PATH_HEADER,
	PATH_PADDING,
	flowDirection,
	layoutCanvas,
	layoutPositions,
	splitPlaced
} from './layout';

describe('layoutPositions', () => {
	it('keeps saved positions exactly', () => {
		const placed = makeNode({ id: 'placed', canvas_x: 300, canvas_y: -20 });
		expect(layoutPositions([placed], []).get('placed')).toEqual({ x: 300, y: -20 });
	});

	it('lays prerequisites left of their dependents', () => {
		const course = makeNode({ id: 'course' });
		const renderer = makeNode({ id: 'renderer' });
		const positions = layoutPositions(
			[course, renderer],
			[makeEdge('renderer', 'course', 'requires')]
		);
		expect(positions.get('course')!.x).toBeLessThan(positions.get('renderer')!.x);
	});

	it('lays a node left of what it precedes', () => {
		const positions = layoutPositions(
			[makeNode({ id: 'later' }), makeNode({ id: 'first' })],
			[makeEdge('first', 'later', 'precedes')]
		);
		expect(positions.get('first')!.x).toBeLessThan(positions.get('later')!.x);
	});

	it('stacks related nodes in one column', () => {
		const positions = layoutPositions(
			[makeNode({ id: 'a' }), makeNode({ id: 'b' })],
			[makeEdge('a', 'b', 'related')]
		);
		expect(positions.get('a')!.x).toBe(positions.get('b')!.x);
		expect(positions.get('a')!.y).not.toBe(positions.get('b')!.y);
	});

	it('parks auto-laid-out nodes below every placed node', () => {
		const placed = makeNode({ id: 'placed', canvas_x: 0, canvas_y: 500 });
		const fresh = makeNode({ id: 'fresh' });
		const positions = layoutPositions([placed, fresh], []);
		expect(positions.get('fresh')!.y).toBeGreaterThan(500 + CANVAS_NODE_HEIGHT);
	});
});

describe('layoutCanvas with collapsed paths', () => {
	it('sizes a collapsed path like a card', () => {
		const path = makeNode({ id: 'path', kind: 'path', canvas_width: 900, canvas_height: 600 });
		const placement = layoutCanvas([path], [], new Set(['path'])).get('path')!;
		expect(placement.size).toEqual({ width: CANVAS_NODE_WIDTH, height: CANVAS_NODE_HEIGHT });
	});
});

describe('flowDirection', () => {
	it('flips requires so the arrow runs prerequisite to dependent', () => {
		expect(flowDirection(makeEdge('dependent', 'prerequisite', 'requires'))).toEqual({
			source: 'prerequisite',
			target: 'dependent'
		});
		expect(flowDirection(makeEdge('child', 'container', 'part_of'))).toEqual({
			source: 'child',
			target: 'container'
		});
	});
});

describe('layoutCanvas', () => {
	it('places children relative to their path and grows the box around them', () => {
		const path = makeNode({ id: 'path', kind: 'path' });
		const inside = makeNode({ id: 'inside' });
		const outside = makeNode({ id: 'outside' });
		const placements = layoutCanvas(
			[path, inside, outside],
			[makeEdge('inside', 'path', 'part_of')]
		);

		const child = placements.get('inside')!;
		expect(child.parentId).toBe('path');
		expect(child.position.x).toBeGreaterThanOrEqual(PATH_PADDING);
		expect(child.position.y).toBeGreaterThanOrEqual(PATH_HEADER);

		const box = placements.get('path')!;
		expect(box.parentId).toBeUndefined();
		expect(box.size.width).toBeGreaterThanOrEqual(child.position.x + CANVAS_NODE_WIDTH);
		expect(box.size.height).toBeGreaterThanOrEqual(child.position.y + CANVAS_NODE_HEIGHT);
		expect(placements.get('outside')!.parentId).toBeUndefined();
	});

	it('keeps a saved box size and nests paths inside paths', () => {
		const outer = makeNode({ id: 'outer', kind: 'path', canvas_width: 900, canvas_height: 600 });
		const inner = makeNode({ id: 'inner', kind: 'path' });
		const leaf = makeNode({ id: 'leaf' });
		const placements = layoutCanvas(
			[outer, inner, leaf],
			[makeEdge('inner', 'outer', 'part_of'), makeEdge('leaf', 'inner', 'part_of')]
		);
		expect(placements.get('outer')!.size).toEqual({ width: 900, height: 600 });
		expect(placements.get('inner')!.parentId).toBe('outer');
		expect(placements.get('leaf')!.parentId).toBe('inner');
		expect(placements.get('inner')!.size.width).toBeGreaterThan(CANVAS_NODE_WIDTH);
	});
});

describe('splitPlaced', () => {
	const at = { canvas_x: 0, canvas_y: 0 };
	const ids = (list: { id: string }[]) => list.map((node) => node.id);

	it('draws placed nodes and sends unplaced ones to the panel', () => {
		const { placed, unplaced } = splitPlaced(
			[makeNode({ id: 'placed', ...at }), makeNode({ id: 'loose' })],
			new Map()
		);
		expect(ids(placed)).toEqual(['placed']);
		expect(ids(unplaced)).toEqual(['loose']);
	});

	it('holds back placed nodes inside an unplaced path', () => {
		const nodes = [
			makeNode({ id: 'path', kind: 'path' }),
			makeNode({ id: 'child', ...at }),
			makeNode({ id: 'orphan', ...at })
		];
		const parentOf = parentPathOf([
			makeEdge('child', 'path', 'part_of'),
			makeEdge('orphan', 'hidden', 'part_of')
		]);
		const { placed, unplaced } = splitPlaced(nodes, parentOf);
		// A path that isn't in the list (hidden) doesn't hold its child back.
		expect(ids(placed)).toEqual(['orphan']);
		expect(ids(unplaced)).toEqual(['path']);
	});

	it('lists unplaced children of placed paths', () => {
		const nodes = [makeNode({ id: 'path', kind: 'path', ...at }), makeNode({ id: 'child' })];
		const { placed, unplaced } = splitPlaced(
			nodes,
			parentPathOf([makeEdge('child', 'path', 'part_of')])
		);
		expect(ids(placed)).toEqual(['path']);
		expect(ids(unplaced)).toEqual(['child']);
	});
});
