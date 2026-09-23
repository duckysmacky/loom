import { describe, expect, it } from 'vitest';
import { makeEdge, makeNode } from './fixtures';
import { dependencyOrder } from './order';

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
