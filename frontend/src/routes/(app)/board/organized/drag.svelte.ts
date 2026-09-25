import { edgesApi, nodesApi } from '$lib/api/endpoints';
import { kindChangeLosses } from '$lib/graph/display';
import type { Grouping } from '$lib/graph/grouping';
import { moveBefore } from '$lib/graph/order';
import { parentPathOf } from '$lib/graph/paths';
import { graph } from '$lib/stores/graph.svelte';
import { notifyError } from '$lib/stores/toasts.svelte';
import type { NodeFocus } from '$lib/types/NodeFocus';
import type { NodeKind } from '$lib/types/NodeKind';
import type { NodeResponse } from '$lib/types/NodeResponse';
import type { NodeStatus } from '$lib/types/NodeStatus';

/** Where a dragged card can land: a top-level group section, or inside a path box. */
export type DropZone =
	{ kind: 'section'; grouping: Grouping; sectionId: string } | { kind: 'path'; pathId: string };

type DropRequest = {
	node: NodeResponse;
	zone: DropZone;
	/** The target zone's card ids, dragged node excluded, in display order. */
	siblingIds: string[];
	beforeId: string | null;
};

/** Which card is being dragged, which slot it's hovering before/after, (for
 * tag grouping, where a card can sit in several sections at once) the tag
 * section it was dragged out of, and a kind change waiting on confirmation
 * (same rule as the NodeDetail/PromoteDialog kind change dialog - it would
 * drop data, so it doesn't apply until the user confirms it). */
export const dragState = $state<{
	draggedId: string | null;
	originSectionId: string | null;
	overId: string | null;
	overBefore: boolean;
	pendingKindDrop: { request: DropRequest; kind: NodeKind } | null;
}>({
	draggedId: null,
	originSectionId: null,
	overId: null,
	overBefore: false,
	pendingKindDrop: null
});

export function startDrag(nodeId: string, originSectionId: string | null = null) {
	dragState.draggedId = nodeId;
	dragState.originSectionId = originSectionId;
}

export function dragOverSlot(slotId: string, event: DragEvent) {
	event.preventDefault();
	const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
	dragState.overId = slotId;
	dragState.overBefore = event.clientX < rect.left + rect.width / 2;
}

export function endDrag() {
	dragState.draggedId = null;
	dragState.originSectionId = null;
	dragState.overId = null;
}

export function requestDrop(request: DropRequest) {
	const { node, zone } = request;
	if (zone.kind === 'section' && zone.grouping === 'kind' && zone.sectionId !== node.kind) {
		const kind = zone.sectionId as NodeKind;
		if (kindChangeLosses(node, kind).length) {
			dragState.pendingKindDrop = { request, kind };
			return;
		}
	}
	applyDrop(request);
}

export function confirmKindDrop() {
	if (dragState.pendingKindDrop) applyDrop(dragState.pendingKindDrop.request);
	dragState.pendingKindDrop = null;
}

export function cancelKindDrop() {
	dragState.pendingKindDrop = null;
}

/** Dropped past every card in a zone - append at the end of it. */
export function dropAtEnd(zone: DropZone, zoneNodeIds: string[]) {
	const draggedId = dragState.draggedId;
	const dragged = draggedId ? graph.nodeById.get(draggedId) : undefined;
	if (!dragged) return;
	requestDrop({
		node: dragged,
		zone,
		siblingIds: zoneNodeIds.filter((id) => id !== draggedId),
		beforeId: null
	});
}

/** Dropped on a specific card - insert before/after it, from the last
 * dragover's side (`dragOverSlot` already recorded which half was hovered). */
export function dropOnCard(target: NodeResponse, zone: DropZone, zoneNodeIds: string[]) {
	const draggedId = dragState.draggedId;
	const dragged = draggedId ? graph.nodeById.get(draggedId) : undefined;
	if (!dragged || draggedId === target.id) return;
	const siblingIds = zoneNodeIds.filter((id) => id !== draggedId);
	const targetIndex = siblingIds.indexOf(target.id);
	const beforeId = dragState.overBefore ? target.id : (siblingIds[targetIndex + 1] ?? null);
	requestDrop({ node: dragged, zone, siblingIds, beforeId });
}

async function applyDrop({ node, zone, siblingIds, beforeId }: DropRequest) {
	const originSectionId = dragState.originSectionId;
	const parentOf = parentPathOf(graph.edges);
	const currentParentId = parentOf.get(node.id) ?? null;
	const targetParentId = zone.kind === 'path' ? zone.pathId : null;

	const result = await graph.mutate(async () => {
		if (targetParentId !== currentParentId) {
			const previousEdge = graph.edges.find(
				(edge) => edge.kind === 'part_of' && edge.from_node_id === node.id
			);
			if (previousEdge) await edgesApi.remove(previousEdge.id);
			if (targetParentId) {
				await edgesApi.create({
					from_node_id: node.id,
					to_node_id: targetParentId,
					kind: 'part_of'
				});
			}
		}

		if (zone.kind === 'section') {
			if (zone.grouping === 'focus' && zone.sectionId !== node.focus) {
				await nodesApi.update(node.id, { focus: zone.sectionId as NodeFocus });
			} else if (zone.grouping === 'status' && zone.sectionId !== node.status) {
				await nodesApi.update(node.id, { status: zone.sectionId as NodeStatus });
			} else if (zone.grouping === 'kind' && zone.sectionId !== node.kind) {
				await nodesApi.update(node.id, { kind: zone.sectionId as NodeKind });
			} else if (zone.grouping === 'tag') {
				if (zone.sectionId === 'untagged') {
					for (const topicId of node.topic_ids) await nodesApi.detachTopic(node.id, topicId);
				} else if (!node.topic_ids.includes(zone.sectionId)) {
					await nodesApi.attachTopic(node.id, zone.sectionId);
				}
				if (
					originSectionId &&
					originSectionId !== 'untagged' &&
					originSectionId !== zone.sectionId
				) {
					await nodesApi.detachTopic(node.id, originSectionId);
				}
			}
		}

		await nodesApi.reorder(moveBefore(siblingIds, node.id, beforeId));
		return true;
	});

	if (!result) return; // graph.mutate already reported the error via a toast
	endDrag();
}

/** Clears every manually-set order, back to the auto (status/dependency) order. */
export async function resetOrder() {
	try {
		await nodesApi.clearOrder();
		await graph.load();
	} catch (error) {
		notifyError(error);
	}
}
