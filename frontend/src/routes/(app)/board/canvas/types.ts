import type { Edge, Node } from '@xyflow/svelte';
import type { EdgeKind } from '$lib/types/EdgeKind';
import type { NodeResponse } from '$lib/types/NodeResponse';

export type LoomFlowNode = Node<{ node: NodeResponse; dimmed: boolean }, 'loom'>;
export type LoomFlowEdge = Edge<{ kind: EdgeKind; unmet: boolean; dimmed: boolean }, 'loom'>;
