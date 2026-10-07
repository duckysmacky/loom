import { isIdea } from '$lib/graph/display';
import { nodeMatches, type FilterCriteria } from '$lib/graph/filters';
import type { NodeResponse } from '$lib/types/NodeResponse';

/**
 * The one filter/search state shared by the Organized, Canvas and Timeline
 * board views - it lives at module level, so switching views keeps it.
 * `showIdeas` isn't part of `FilterCriteria` (the generic node-matching
 * predicate doesn't know about ideas) - it's a board-only on/off switch
 * layered on top, same as the idea exclusion it overrides.
 */
export const boardFilters = $state<FilterCriteria & { showIdeas: boolean; showDone: boolean }>({
	search: '',
	kinds: [],
	statuses: [],
	focuses: [],
	topicIds: [],
	showArchived: false,
	showIdeas: false,
	showDone: false
});

/** Board views hide ideas unless the "Show ideas" switch is on;
 * beyond that, the shared filters apply. */
export function matchesBoardFilters(node: NodeResponse): boolean {
	return (boardFilters.showIdeas || !isIdea(node)) && nodeMatches(node, boardFilters);
}

/** The Organized view also hides done nodes unless "Show done" is on or
 * the status filter asks for them. */
export function matchesOrganizedFilters(node: NodeResponse): boolean {
	return (
		(boardFilters.showDone || node.status !== 'done' || boardFilters.statuses.includes('done')) &&
		matchesBoardFilters(node)
	);
}

export function clearBoardFilters() {
	boardFilters.search = '';
	boardFilters.kinds = [];
	boardFilters.statuses = [];
	boardFilters.focuses = [];
	boardFilters.topicIds = [];
	boardFilters.showArchived = false;
	boardFilters.showIdeas = false;
	boardFilters.showDone = false;
}
