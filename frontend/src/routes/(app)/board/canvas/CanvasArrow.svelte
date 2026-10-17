<script lang="ts">
	import { BaseEdge, Position, getBezierPath, type EdgeProps } from '@xyflow/svelte';
	import type { LoomArrowEdge } from './types';

	let {
		id,
		sourceX,
		sourceY,
		targetX,
		targetY,
		sourcePosition,
		targetPosition,
		selected
	}: EdgeProps<LoomArrowEdge> = $props();

	const geometry = $derived(
		getBezierPath({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition })
	);
	// Open chevron (not the filled graph arrowhead), arms pointing back out of
	// whichever side the arrow enters.
	const arrowHead = $derived.by(() => {
		const [backX, backY] = {
			[Position.Left]: [-1, 0],
			[Position.Right]: [1, 0],
			[Position.Top]: [0, -1],
			[Position.Bottom]: [0, 1]
		}[targetPosition];
		const armX = targetX + backX * 10;
		const armY = targetY + backY * 10;
		// The arms spread across the line's direction.
		return `M ${armX - backY * 6} ${armY - backX * 6} L ${targetX} ${targetY} L ${armX + backY * 6} ${armY + backX * 6}`;
	});
</script>

<!-- An annotation arrow: a hand-drawn-looking dashed line, no graph meaning. -->
<g class="arrow-edge" class:selected>
	<BaseEdge {id} path={geometry[0]} interactionWidth={16} />
	<path class="head" d={arrowHead} />
</g>

<style>
	.arrow-edge :global(.svelte-flow__edge-path),
	.head {
		fill: none;
		stroke: var(--note-edge);
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.arrow-edge :global(.svelte-flow__edge-path) {
		stroke-dasharray: 7 5;
	}

	.selected :global(.svelte-flow__edge-path),
	.selected .head {
		stroke: var(--accent);
	}
</style>
