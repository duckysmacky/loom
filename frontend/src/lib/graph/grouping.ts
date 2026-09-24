import type { NodeResponse } from '$lib/types/NodeResponse';
import type { TopicResponse } from '$lib/types/TopicResponse';
import { KIND_GLYPH, TIER_COLOR } from './display';

export type Grouping = 'focus' | 'kind' | 'status' | 'tag';

export type Section = { id: string; title: string; bar: string; nodes: NodeResponse[] };

type Bucket = { id: string; title: string; bar: string; holds: (node: NodeResponse) => boolean };

const FOCUS_BUCKETS: Bucket[] = (['primary', 'secondary', 'background'] as const).map((focus) => ({
	id: focus,
	title: focus,
	bar: TIER_COLOR[focus],
	holds: (node) => node.focus === focus
}));

const KIND_BUCKETS: Bucket[] = (
	[
		['project', 'projects', 'var(--node-blue)'],
		['study', 'studies', 'var(--node-course)'],
		['path', 'paths', 'var(--node-path)'],
		['idea', 'ideas', 'var(--node-idea)']
	] as const
).map(([kind, title, bar]) => ({
	id: kind,
	title: `${KIND_GLYPH[kind]} ${title}`,
	bar,
	holds: (node) => node.kind === kind
}));

const STATUS_BUCKETS: Bucket[] = (
	[
		['active', 'var(--accent)'],
		['queued', 'var(--ink-2)'],
		['paused', 'var(--ink-2)'],
		['idea', 'var(--line)'],
		['done', 'var(--ok)'],
		['archived', 'var(--line)']
	] as const
).map(([status, bar]) => ({
	id: status,
	title: status,
	bar,
	holds: (node) => node.status === status
}));

function tagBuckets(topics: TopicResponse[]): Bucket[] {
	return [
		...topics
			.toSorted((left, right) => left.name.localeCompare(right.name))
			.map((topic) => ({
				id: topic.id,
				title: topic.name,
				bar: topic.color ?? 'var(--line)',
				holds: (node: NodeResponse) => node.topic_ids.includes(topic.id)
			})),
		{
			id: 'untagged',
			title: 'untagged',
			bar: 'var(--line)',
			holds: (node) => !node.topic_ids.length
		}
	];
}

/**
 * Splits already-ordered nodes into the Organized view's sections, keeping
 * their order within each section and dropping empty sections. With tags a
 * node appears under every tag it has.
 */
export function groupNodes(
	nodes: NodeResponse[],
	grouping: Grouping,
	topics: TopicResponse[] = []
): Section[] {
	const buckets = {
		focus: FOCUS_BUCKETS,
		kind: KIND_BUCKETS,
		status: STATUS_BUCKETS,
		tag: tagBuckets(topics)
	}[grouping];
	return buckets
		.map(({ id, title, bar, holds }) => ({ id, title, bar, nodes: nodes.filter(holds) }))
		.filter((section) => section.nodes.length > 0);
}
