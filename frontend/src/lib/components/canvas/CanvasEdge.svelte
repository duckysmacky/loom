<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { BaseEdge, getBezierPath, type EdgeProps } from '@xyflow/svelte';
	import { arrowPath, edgeStrokeProps, edgeStyleKey, type HandleSide } from '$lib/graph/edgeStyle';
	import { prefs } from '$lib/stores/prefs.svelte';
	import type { LoomGraphEdge } from './types';

	let {
		id,
		sourceX,
		sourceY,
		targetX,
		targetY,
		sourcePosition,
		targetPosition,
		data,
		selected
	}: EdgeProps<LoomGraphEdge> = $props();

	const kind = $derived(data?.kind ?? 'related');
	const unmet = $derived(data?.unmet ?? false);

	const geometry = $derived(
		getBezierPath({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition })
	);
	// Settings > Appearance > Connections; part_of (the path box) has no style.
	const styleKey = $derived(edgeStyleKey(kind, unmet));
	const edgeStyle = $derived(styleKey ? prefs.edgeStyles[styleKey] : null);
	const stroke = $derived(edgeStyle ? edgeStrokeProps(edgeStyle) : null);
	const arrowHead = $derived(arrowPath(targetX, targetY, targetPosition as HandleSide));
	// Optional (Settings > Appearance): whether a requirement is met already
	// shows in the line's colour, so the label only names the connection.
	const label = $derived(
		prefs.showEdgeLabels
			? {
					requires: m.edge_label_requires,
					part_of: m.edge_label_part_of,
					related: m.edge_label_related,
					precedes: m.edge_label_next
				}[kind]()
			: null
	);
</script>

<!-- requires: strong directional line (orange while unmet, green once met);
part_of: a heavy double rail into the container, reading as nesting; related
and precedes: lighter dashed lines (precedes arrowed, an order that never
blocks). Colour, line, width and arrow come from the user's connection styles. -->
<g
	class="edge {kind} {styleKey ?? ''}"
	class:unmet
	class:dimmed={data?.dimmed}
	class:selected
	style:--edge-stroke={stroke?.stroke}
	style:--edge-width="{stroke?.width ?? 2}px"
	style:--edge-dash={stroke?.dasharray}
	style:--edge-cap={stroke?.linecap}
>
	{#if kind === 'part_of'}
		<path class="rail-outer" d={geometry[0]} />
	{/if}
	<BaseEdge {id} path={geometry[0]} interactionWidth={16} />
	{#if edgeStyle?.arrow}
		<path class="arrow" d={arrowHead} />
	{/if}
	{#if kind === 'part_of'}
		<rect class="socket" x={targetX - 6} y={targetY - 6} width="6" height="12" />
	{/if}
</g>
{#if label}
	<text class="edge-label" class:unmet x={geometry[1]} y={geometry[2] - 6} text-anchor="middle"
		>{label}</text
	>
{/if}

<style>
	/* Token defaults per style; an explicit colour (--edge-stroke) wins. */
	.related {
		--edge-default: var(--ink-2);
	}

	.precedes {
		--edge-default: var(--edge-precedes);
	}

	.requires_unmet {
		--edge-default: var(--warn);
	}

	.requires_met {
		--edge-default: var(--ok);
	}

	.edge :global(.svelte-flow__edge-path) {
		fill: none;
		stroke: var(--edge-stroke, var(--edge-default, var(--ink-2)));
		stroke-width: var(--edge-width);
		stroke-dasharray: var(--edge-dash, none);
		stroke-linecap: var(--edge-cap, butt);
	}

	.arrow {
		fill: var(--edge-stroke, var(--edge-default, var(--ink-2)));
	}

	.rail-outer {
		fill: none;
		stroke: var(--node-path);
		stroke-width: 7;
	}

	.part_of :global(.svelte-flow__edge-path) {
		stroke: var(--bg);
		stroke-width: 2.5;
		stroke-dasharray: none;
	}

	.socket {
		fill: var(--node-path);
	}

	.selected:not(.part_of) :global(.svelte-flow__edge-path),
	.selected .rail-outer {
		stroke: var(--accent);
	}

	.selected .arrow {
		fill: var(--accent);
	}

	.dimmed {
		opacity: 0.2;
	}

	.edge-label {
		font: 700 10px/1 var(--font-mono);
		fill: var(--ink-2);
		paint-order: stroke;
		stroke: var(--bg);
		stroke-width: 4px;
	}

	.edge-label.unmet {
		fill: var(--warn);
	}
</style>
