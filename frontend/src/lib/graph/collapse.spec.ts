import { describe, expect, it } from 'vitest';
import { rerouteEdges, standIns } from './collapse';
import { makeEdge } from './fixtures';
import { parentPathOf } from './paths';

// outer ⊃ inner ⊃ leaf; sibling sits in outer next to inner.
const containment = [
	makeEdge('inner', 'outer', 'part_of'),
	makeEdge('leaf', 'inner', 'part_of'),
	makeEdge('sibling', 'outer', 'part_of')
];
const parentOf = parentPathOf(containment);
const ids = ['outer', 'inner', 'leaf', 'sibling', 'free'];

describe('standIns', () => {
	it('hides everything inside a collapsed path behind the outermost collapsed one', () => {
		const standIn = standIns(ids, parentOf, new Set(['outer', 'inner']));
		expect(Object.fromEntries(standIn)).toEqual({
			inner: 'outer',
			leaf: 'outer',
			sibling: 'outer'
		});
	});

	it('leaves expanded paths alone', () => {
		const standIn = standIns(ids, parentOf, new Set(['inner']));
		expect(Object.fromEntries(standIn)).toEqual({ leaf: 'inner' });
	});
});

describe('rerouteEdges', () => {
	const standIn = standIns(ids, parentOf, new Set(['outer']));

	it('draws edges to hidden nodes from the collapsed path, merging duplicates', () => {
		const routed = rerouteEdges(
			[makeEdge('free', 'leaf', 'requires'), makeEdge('free', 'sibling', 'requires')],
			standIn
		);
		expect(routed).toHaveLength(1);
		expect(routed[0].edge.to_node_id).toBe('outer');
		expect(routed[0].merged).toHaveLength(2);
		expect(routed[0].rerouted).toBe(true);
	});

	it('drops edges that end up inside one collapsed path, and part_of', () => {
		const routed = rerouteEdges([makeEdge('leaf', 'sibling', 'related'), ...containment], standIn);
		expect(routed).toEqual([]);
	});

	it('keeps untouched edges as they are', () => {
		const edge = makeEdge('free', 'outer', 'related');
		expect(rerouteEdges([edge], standIn)).toEqual([{ edge, merged: [edge], rerouted: false }]);
	});
});
