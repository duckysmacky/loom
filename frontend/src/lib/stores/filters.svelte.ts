import { isBacklog } from '$lib/graph/display';
import { nodeMatches, type FilterCriteria } from '$lib/graph/filters';
import type { NodeResponse } from '$lib/types/NodeResponse';

/**
 * The one filter/search state shared by the Organized, Canvas and Timeline
 * board views - it lives at module level, so switching views keeps it.
 * `showBacklog` isn't part of `FilterCriteria` (the generic node-matching
 * predicate doesn't know about backlog) - it's a board-only on/off switch
 * layered on top, same as the backlog exclusion it overrides.
 */
export const boardFilters = $state<FilterCriteria & { showBacklog: boolean }>({
	search: '',
	kinds: [],
	statuses: [],
	focuses: [],
	topicIds: [],
	showArchived: false,
	showBacklog: false
});

/** Board views hide backlog ideas unless the "Show backlog" switch is on;
 * beyond that, the shared filters apply. */
export function matchesBoardFilters(node: NodeResponse): boolean {
	return (boardFilters.showBacklog || !isBacklog(node)) && nodeMatches(node, boardFilters);
}

export function clearBoardFilters() {
	boardFilters.search = '';
	boardFilters.kinds = [];
	boardFilters.statuses = [];
	boardFilters.focuses = [];
	boardFilters.topicIds = [];
	boardFilters.showArchived = false;
	boardFilters.showBacklog = false;
}
