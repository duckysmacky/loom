import { describe, expect, it } from 'vitest';
import { makeEdge, makeNode } from './fixtures';
import {
	CANVAS_NODE_HEIGHT,
	CANVAS_NODE_WIDTH,
	PATH_HEADER,
	PATH_PADDING,
	flowDirection,
	layoutCanvas,
	layoutPositions
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

	it('parks auto-laid-out nodes below every placed node', () => {
		const placed = makeNode({ id: 'placed', canvas_x: 0, canvas_y: 500 });
		const fresh = makeNode({ id: 'fresh' });
		const positions = layoutPositions([placed, fresh], []);
		expect(positions.get('fresh')!.y).toBeGreaterThan(500 + CANVAS_NODE_HEIGHT);
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
