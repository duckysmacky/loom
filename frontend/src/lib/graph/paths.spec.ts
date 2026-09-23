import { describe, expect, it } from 'vitest';
import { makeEdge, makeNode } from './fixtures';
import { ancestorPaths, matchesOrContainsMatch, nestingDepth, parentPathOf } from './paths';

describe('path containment', () => {
	const edges = [
		makeEdge('lesson', 'module', 'part_of'),
		makeEdge('module', 'course-path', 'part_of'),
		makeEdge('lesson', 'other', 'requires')
	];
	const parentOf = parentPathOf(edges);

	it('maps each child to its path, ignoring other edge kinds', () => {
		expect([...parentOf]).toEqual([
			['lesson', 'module'],
			['module', 'course-path']
		]);
	});

	it('walks nested paths innermost first', () => {
		expect(ancestorPaths('lesson', parentOf)).toEqual(['module', 'course-path']);
		expect(nestingDepth('lesson', parentOf)).toBe(2);
		expect(nestingDepth('course-path', parentOf)).toBe(0);
	});

	it('survives a cycle', () => {
		const cyclic = parentPathOf([makeEdge('a', 'b', 'part_of'), makeEdge('b', 'a', 'part_of')]);
		expect(ancestorPaths('a', cyclic)).toEqual(['b', 'a']);
	});
});

describe('matchesOrContainsMatch', () => {
	it('keeps a path visible while something nested inside it matches', () => {
		const outer = makeNode({ id: 'outer', kind: 'path', title: 'Outer' });
		const inner = makeNode({ id: 'inner', kind: 'path', title: 'Inner' });
		const leaf = makeNode({ id: 'leaf', title: 'Needle' });
		const nodes = [outer, inner, leaf];
		const parentOf = parentPathOf([
			makeEdge('inner', 'outer', 'part_of'),
			makeEdge('leaf', 'inner', 'part_of')
		]);
		const needle = (node: { title: string }) => node.title === 'Needle';
		expect(matchesOrContainsMatch(outer, nodes, parentOf, needle)).toBe(true);
		expect(matchesOrContainsMatch(outer, nodes, parentOf, () => false)).toBe(false);
	});
});
