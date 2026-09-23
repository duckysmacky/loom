<script lang="ts">
	import { BaseEdge, getBezierPath, type EdgeProps } from '@xyflow/svelte';
	import type { LoomFlowEdge } from './types';

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
	}: EdgeProps<LoomFlowEdge> = $props();

	const kind = $derived(data?.kind ?? 'related');
	const unmet = $derived(data?.unmet ?? false);

	const geometry = $derived(
		getBezierPath({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition })
	);
	const arrowHead = $derived(
		`M ${targetX} ${targetY} L ${targetX - 9} ${targetY - 5} L ${targetX - 9} ${targetY + 5} Z`
	);
	// Unmet requirements are always labelled; other edges only when selected.
	const label = $derived(
		kind === 'requires' && unmet
			? 'requires · unmet'
			: selected
				? { requires: 'requires · met', part_of: 'part of', related: 'related' }[kind]
				: null
	);
</script>

<!-- requires: strong directional line (warn while unmet); part_of: a heavy
double rail into the container, reading as nesting; related: light dots. -->
<g class="edge {kind}" class:unmet class:dimmed={data?.dimmed} class:selected>
	{#if kind === 'part_of'}
		<path class="rail-outer" d={geometry[0]} />
	{/if}
	<BaseEdge {id} path={geometry[0]} interactionWidth={16} />
	{#if kind === 'requires'}
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
	.edge :global(.svelte-flow__edge-path) {
		fill: none;
		stroke: var(--ink-2);
		stroke-width: 2;
	}

	.requires.unmet :global(.svelte-flow__edge-path) {
		stroke: var(--warn);
		stroke-width: 2.5;
	}

	.arrow {
		fill: var(--ink-2);
	}

	.unmet .arrow {
		fill: var(--warn);
	}

	.rail-outer {
		fill: none;
		stroke: var(--node-path);
		stroke-width: 7;
	}

	.part_of :global(.svelte-flow__edge-path) {
		stroke: var(--bg);
		stroke-width: 2.5;
	}

	.socket {
		fill: var(--node-path);
	}

	.related :global(.svelte-flow__edge-path) {
		stroke-width: 1.5;
		stroke-dasharray: 2 5;
		stroke-linecap: round;
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
</style>
