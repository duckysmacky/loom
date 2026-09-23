import { describe, expect, it } from 'vitest';
import { nodeMatches, type FilterCriteria } from './filters';
import { makeNode } from './fixtures';

const anything: FilterCriteria = {
	search: '',
	kinds: [],
	statuses: [],
	focuses: [],
	topicIds: [],
	showArchived: false
};

describe('nodeMatches', () => {
	it('matches everything but archived nodes by default', () => {
		expect(nodeMatches(makeNode(), anything)).toBe(true);
		expect(nodeMatches(makeNode({ status: 'archived' }), anything)).toBe(false);
		expect(nodeMatches(makeNode({ status: 'archived' }), { ...anything, showArchived: true })).toBe(
			true
		);
		expect(
			nodeMatches(makeNode({ status: 'archived' }), { ...anything, statuses: ['archived'] })
		).toBe(true);
	});

	it('searches titles case-insensitively', () => {
		const node = makeNode({ title: 'Custom OpenGL renderer' });
		expect(nodeMatches(node, { ...anything, search: 'opengl' })).toBe(true);
		expect(nodeMatches(node, { ...anything, search: 'vulkan' })).toBe(false);
	});

	it('filters kinds by display kind, so containers match "path"', () => {
		const container = makeNode({ kind: 'project', container_progress: { done: 0, total: 2 } });
		expect(nodeMatches(container, { ...anything, kinds: ['path'] })).toBe(true);
		expect(nodeMatches(container, { ...anything, kinds: ['project'] })).toBe(false);
	});

	it('ANDs criteria and ORs values within one criterion', () => {
		const node = makeNode({ focus: 'primary', topic_ids: ['graphics'] });
		expect(nodeMatches(node, { ...anything, focuses: ['primary', 'secondary'] })).toBe(true);
		expect(nodeMatches(node, { ...anything, focuses: ['primary'], topicIds: ['infra'] })).toBe(
			false
		);
	});
});
