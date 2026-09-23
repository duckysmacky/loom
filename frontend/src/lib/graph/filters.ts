import type { NodeFocus } from '$lib/types/NodeFocus';
import type { NodeKind } from '$lib/types/NodeKind';
import type { NodeResponse } from '$lib/types/NodeResponse';
import type { NodeStatus } from '$lib/types/NodeStatus';

/** Board filter criteria. Empty lists mean "any". */
export type FilterCriteria = {
	search: string;
	kinds: NodeKind[];
	statuses: NodeStatus[];
	focuses: NodeFocus[];
	topicIds: string[];
	showArchived: boolean;
};

export function nodeMatches(node: NodeResponse, criteria: FilterCriteria): boolean {
	const search = criteria.search.trim().toLowerCase();
	if (search && !node.title.toLowerCase().includes(search)) return false;
	if (
		!criteria.showArchived &&
		node.status === 'archived' &&
		!criteria.statuses.includes('archived')
	)
		return false;
	if (criteria.kinds.length && !criteria.kinds.includes(node.kind)) return false;
	if (criteria.statuses.length && !criteria.statuses.includes(node.status)) return false;
	if (criteria.focuses.length && !criteria.focuses.includes(node.focus)) return false;
	if (criteria.topicIds.length && !criteria.topicIds.some((id) => node.topic_ids.includes(id)))
		return false;
	return true;
}
