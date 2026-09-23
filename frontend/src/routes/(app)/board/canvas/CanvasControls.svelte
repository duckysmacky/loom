<script lang="ts">
	import { Panel, useSvelteFlow } from '@xyflow/svelte';
	import { page } from '$app/state';
	import { CANVAS_NODE_HEIGHT, CANVAS_NODE_WIDTH } from '$lib/graph/layout';

	let { zoom }: { zoom: number } = $props();

	const flow = useSvelteFlow();

	const MIN_ZOOM_PERCENT = 20;
	const MAX_ZOOM_PERCENT = 200;

	// Typed zoom: the field follows the live zoom until the user edits it.
	let zoomDraft = $derived(String(Math.round(zoom * 100)));

	function applyZoom() {
		const percent = Number.parseFloat(zoomDraft);
		if (!Number.isFinite(percent)) {
			zoomDraft = String(Math.round(zoom * 100));
			return;
		}
		const clamped = Math.min(MAX_ZOOM_PERCENT, Math.max(MIN_ZOOM_PERCENT, percent));
		zoomDraft = String(Math.round(clamped));
		flow.setZoom(clamped / 100, { duration: 200 });
	}

	// "Open in canvas" deep link: /board/canvas?focus=<node id> centres on it.
	$effect(() => {
		const focusId = page.url.searchParams.get('focus');
		const target = focusId ? flow.getNode(focusId) : undefined;
		if (target) {
			flow.setCenter(
				target.position.x + CANVAS_NODE_WIDTH / 2,
				target.position.y + CANVAS_NODE_HEIGHT / 2,
				{ zoom: 1.2, duration: 300 }
			);
		}
	});
</script>

<Panel position="bottom-left">
	<div class="controls">
		<button type="button" aria-label="Zoom out" onclick={() => flow.zoomOut()}>−</button>
		<label class="zoom">
			<input
				type="text"
				inputmode="numeric"
				aria-label="Zoom percent"
				bind:value={zoomDraft}
				onkeydown={(event) => event.key === 'Enter' && event.currentTarget.blur()}
				onblur={applyZoom}
			/>%
		</label>
		<button type="button" aria-label="Zoom in" onclick={() => flow.zoomIn()}>+</button>
		<button type="button" class="fit" onclick={() => flow.fitView({ duration: 300 })}>Fit</button>
		<span class="hint">drag to pan · scroll to zoom · handle to connect</span>
	</div>
</Panel>

<Panel position="bottom-right">
	<div class="legend">
		<div class="label">Edges</div>
		<div class="entry"><span class="swatch requires-unmet"></span>requires · unmet</div>
		<div class="entry"><span class="swatch requires-met"></span>requires · met</div>
		<div class="entry"><span class="swatch part-of"></span>part of (child → path)</div>
		<div class="entry"><span class="swatch related"></span>related</div>
	</div>
</Panel>

<style>
	.controls,
	.legend {
		background: var(--surface);
		border: var(--border-width-hair) solid var(--line);
		font-family: var(--font-display);
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
	}

	button {
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		color: var(--ink);
		font: 700 12px/1 var(--font-mono);
		min-width: 24px;
		height: 24px;
	}

	.fit {
		padding: 0 8px;
	}

	.zoom {
		display: inline-flex;
		align-items: center;
		gap: 1px;
		font: 700 11px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.zoom input {
		width: 38px;
		height: 24px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface-2);
		text-align: right;
		padding: 0 4px;
		font: 700 11px/1 var(--font-mono);
		color: var(--ink);
	}

	.zoom input:focus {
		outline: none;
		border-color: var(--accent);
		background: var(--surface);
	}

	.hint {
		margin-left: 4px;
		font: 600 11.5px/1 var(--font-display);
		color: var(--ink-2);
	}

	.legend {
		padding: 11px 13px;
	}

	.legend .entry {
		margin-top: 8px;
		display: flex;
		align-items: center;
		gap: 8px;
		font: 600 11px/1 var(--font-display);
		color: var(--ink-3);
	}

	.swatch {
		width: 26px;
		display: inline-block;
	}

	.requires-unmet {
		height: 3px;
		background: var(--warn);
	}

	.requires-met {
		height: 2px;
		background: var(--ink-2);
	}

	.part-of {
		height: 7px;
		background: linear-gradient(
			var(--node-path) 0 35%,
			var(--bg) 35% 65%,
			var(--node-path) 65% 100%
		);
	}

	.related {
		border-top: 2px dotted var(--ink-2);
	}

	@media (max-width: 700px) {
		.hint {
			display: none;
		}
	}
</style>
