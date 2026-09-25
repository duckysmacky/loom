import { describe, expect, it } from 'vitest';
import { makeEdge, makeNode } from './fixtures';
import { dependencyOrder, manualOrder, moveBefore } from './order';

const ids = (nodes: { id: string }[]) => nodes.map((node) => node.id);

describe('dependencyOrder', () => {
	it('puts a blocked node right after the node it requires', () => {
		const nodes = ['renderer', 'axum', 'opengl'].map((id) => makeNode({ id }));
		const edges = [makeEdge('renderer', 'opengl', 'requires')];
		expect(ids(dependencyOrder(nodes, edges))).toEqual(['opengl', 'renderer', 'axum']);
	});

	it('follows chains and keeps the original order otherwise', () => {
		const nodes = ['engine', 'renderer', 'course', 'other'].map((id) => makeNode({ id }));
		const edges = [
			makeEdge('engine', 'renderer', 'requires'),
			makeEdge('renderer', 'course', 'requires'),
			makeEdge('other', 'course', 'related')
		];
		expect(ids(dependencyOrder(nodes, edges))).toEqual(['course', 'renderer', 'engine', 'other']);
	});

	it('ignores requirements outside the list and survives cycles', () => {
		const nodes = ['a', 'b'].map((id) => makeNode({ id }));
		const edges = [
			makeEdge('a', 'elsewhere', 'requires'),
			makeEdge('a', 'b', 'requires'),
			makeEdge('b', 'a', 'requires')
		];
		expect(ids(dependencyOrder(nodes, edges))).toEqual(['b', 'a']);
	});
});

describe('manualOrder', () => {
	it('sorts ranked nodes by rank, unranked nodes keep incoming order after them', () => {
		const nodes = [
			makeNode({ id: 'c', sort_order: null }),
			makeNode({ id: 'a', sort_order: 2 }),
			makeNode({ id: 'd', sort_order: null }),
			makeNode({ id: 'b', sort_order: 1 })
		];
		expect(ids(manualOrder(nodes))).toEqual(['b', 'a', 'c', 'd']);
	});

	it('is a no-op when nothing is ranked', () => {
		const nodes = ['a', 'b'].map((id) => makeNode({ id }));
		expect(ids(manualOrder(nodes))).toEqual(['a', 'b']);
	});
});

describe('moveBefore', () => {
	it('moves the dragged id before the target', () => {
		expect(moveBefore(['a', 'b', 'c'], 'c', 'a')).toEqual(['c', 'a', 'b']);
	});

	it('appends at the end when beforeId is null', () => {
		expect(moveBefore(['a', 'b', 'c'], 'a', null)).toEqual(['b', 'c', 'a']);
	});

	it('drops the dragged id out of its old slot first', () => {
		expect(moveBefore(['a', 'b', 'c'], 'a', 'c')).toEqual(['b', 'a', 'c']);
	});
});
