import type { Edge, Node } from '@xyflow/svelte';
import type { CanvasNoteResponse } from '$lib/types/CanvasNoteResponse';
import type { EdgeKind } from '$lib/types/EdgeKind';
import type { NodeResponse } from '$lib/types/NodeResponse';

/** `loom` is a regular card; `loomPath` is a path box other nodes sit inside. */
export type LoomCardNode = Node<{ node: NodeResponse; dimmed: boolean }, 'loom' | 'loomPath'>;
/** A sticky note: free text on the canvas, not a graph node. */
export type LoomNoteNode = Node<{ note: CanvasNoteResponse }, 'loomNote'>;
export type LoomFlowNode = LoomCardNode | LoomNoteNode;

/** `loom` is a graph edge; `loomArrow` an annotation arrow with no graph meaning. */
export type LoomGraphEdge = Edge<{ kind: EdgeKind; unmet: boolean; dimmed: boolean }, 'loom'>;
export type LoomArrowEdge = Edge<Record<string, never>, 'loomArrow'>;
export type LoomFlowEdge = LoomGraphEdge | LoomArrowEdge;

export const isCard = (flowNode: LoomFlowNode): flowNode is LoomCardNode =>
	flowNode.type !== 'loomNote';
