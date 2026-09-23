import { describe, expect, it } from 'vitest';
import { makeEdge, makeNode } from './fixtures';
import { CANVAS_NODE_HEIGHT, flowDirection, layoutPositions } from './layout';

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
