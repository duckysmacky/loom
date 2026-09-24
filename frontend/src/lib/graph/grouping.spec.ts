import { describe, expect, it } from 'vitest';
import type { TopicResponse } from '$lib/types/TopicResponse';
import { makeNode } from './fixtures';
import { groupNodes } from './grouping';

const topic = (id: string, name: string): TopicResponse => ({
	id,
	name,
	color: null,
	created_at: '2026-09-01T00:00:00Z',
	updated_at: '2026-09-01T00:00:00Z'
});

const nodes = [
	makeNode({ id: 'a', kind: 'project', focus: 'primary', status: 'active', topic_ids: ['rust'] }),
	makeNode({ id: 'b', kind: 'study', focus: 'secondary', status: 'queued', topic_ids: [] }),
	makeNode({
		id: 'c',
		kind: 'project',
		focus: 'primary',
		status: 'done',
		topic_ids: ['rust', 'math']
	})
];
const shape = (sections: ReturnType<typeof groupNodes>) =>
	sections.map((section) => [section.id, section.nodes.map((node) => node.id)]);

describe('groupNodes', () => {
	it('groups by focus tier in tier order, dropping empty tiers', () => {
		expect(shape(groupNodes(nodes, 'focus'))).toEqual([
			['primary', ['a', 'c']],
			['secondary', ['b']]
		]);
	});

	it('groups by kind and by status', () => {
		expect(shape(groupNodes(nodes, 'kind'))).toEqual([
			['project', ['a', 'c']],
			['study', ['b']]
		]);
		expect(shape(groupNodes(nodes, 'status'))).toEqual([
			['active', ['a']],
			['queued', ['b']],
			['done', ['c']]
		]);
	});

	it('groups by tag alphabetically, repeating multi-tag nodes, with an untagged bucket', () => {
		const sections = groupNodes(nodes, 'tag', [topic('rust', 'rust'), topic('math', 'math')]);
		expect(shape(sections)).toEqual([
			['math', ['c']],
			['rust', ['a', 'c']],
			['untagged', ['b']]
		]);
	});
});
