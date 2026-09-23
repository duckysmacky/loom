import { nodeMatches, type FilterCriteria } from '$lib/graph/filters';
import type { NodeResponse } from '$lib/types/NodeResponse';

/**
 * The one filter/search state shared by the Organized, Canvas and Timeline
 * board views - it lives at module level, so switching views keeps it.
 */
export const boardFilters = $state<FilterCriteria>({
	search: '',
	kinds: [],
	statuses: [],
	focuses: [],
	topicIds: [],
	showArchived: false
});

export function matchesBoardFilters(node: NodeResponse): boolean {
	return nodeMatches(node, boardFilters);
}

export function clearBoardFilters() {
	boardFilters.search = '';
	boardFilters.kinds = [];
	boardFilters.statuses = [];
	boardFilters.focuses = [];
	boardFilters.topicIds = [];
	boardFilters.showArchived = false;
}
