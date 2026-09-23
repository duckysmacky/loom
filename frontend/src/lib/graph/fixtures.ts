import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';

/** Test helper: a node with sensible defaults, overridable per field. */
export function makeNode(overrides: Partial<NodeResponse> = {}): NodeResponse {
	return {
		id: 'node',
		kind: 'project',
		status: 'active',
		focus: 'secondary',
		title: 'Node',
		progress_current: null,
		progress_total: null,
		progress_unit: null,
		color: null,
		notes: null,
		created_at: '2026-09-01T00:00:00Z',
		updated_at: '2026-09-01T00:00:00Z',
		started_at: null,
		completed_at: null,
		canvas_x: null,
		canvas_y: null,
		canvas_width: null,
		canvas_height: null,
		topic_ids: [],
		blocked: false,
		container_progress: null,
		checklist_progress: null,
		last_poked_at: null,
		...overrides
	};
}

export function makeEdge(
	from: string,
	to: string,
	kind: EdgeResponse['kind'],
	id = `${from}-${kind}-${to}`
): EdgeResponse {
	return { id, from_node_id: from, to_node_id: to, kind, created_at: '2026-09-01T00:00:00Z' };
}
