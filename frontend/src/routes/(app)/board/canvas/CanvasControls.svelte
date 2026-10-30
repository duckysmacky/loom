<script lang="ts">
	import { Panel, useNodesInitialized, useSvelteFlow } from '@xyflow/svelte';
	import { page } from '$app/state';
	import { CANVAS_NODE_HEIGHT, CANVAS_NODE_WIDTH } from '$lib/graph/layout';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';

	let { zoom, onaddnote }: { zoom: number; onaddnote: () => void } = $props();

	const flow = useSvelteFlow();
	const initialized = useNodesInitialized();

	const MIN_ZOOM_PERCENT = 1;
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

	// "Open in canvas" deep link: /board/canvas?focus=<node id> centres on it
	// once. Other URL changes (opening a node's detail sets ?node=) and graph
	// updates must not move the camera again.
	const focusId = $derived(page.url.searchParams.get('focus'));
	let centredOn: string | null = null;
	$effect(() => {
		if (!focusId || focusId === centredOn) return;
		// Waits (reruns) until the node is on the canvas; absolute position,
		// since a node inside a path stores one relative to its box.
		// After the first measurement, so the initial fit-view doesn't override it.
		if (!initialized.current || !flow.getNode(focusId)) return;
		const target = flow.getInternalNode(focusId);
		if (!target) return;
		centredOn = focusId;
		const { x, y } = target.internals.positionAbsolute;
		requestAnimationFrame(() =>
			flow.setCenter(x + CANVAS_NODE_WIDTH / 2, y + CANVAS_NODE_HEIGHT / 2, {
				zoom: 1.2,
				duration: 300
			})
		);
	});
</script>

<Panel position="bottom-left">
	<button type="button" class="add-note" title="Add a sticky note" onclick={onaddnote}
		>+ Note</button
	>
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
		<button
			type="button"
			class="snap"
			aria-pressed={prefs.canvasSnapToGrid}
			title="Snap to grid"
			onclick={() => {
				prefs.canvasSnapToGrid = !prefs.canvasSnapToGrid;
				savePrefs();
			}}>Snap to grid</button
		>
		<span class="hint">drag to pan · shift-drag to select · scroll to zoom · handle to connect</span
		>
	</div>
</Panel>

<Panel position="bottom-right">
	<div class="legend">
		<div class="label">Edges</div>
		<div class="entry"><span class="swatch requires-unmet"></span>requires · unmet</div>
		<div class="entry"><span class="swatch requires-met"></span>requires · met</div>
		<div class="entry"><span class="swatch path-box"></span>path · drop nodes inside</div>
		<div class="entry"><span class="swatch precedes"></span>precedes · do first</div>
		<div class="entry"><span class="swatch related"></span>related</div>
		<div class="entry"><span class="swatch annotation"></span>arrow · annotation only</div>
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

	.fit,
	.snap {
		padding: 0 8px;
	}

	.snap[aria-pressed='true'] {
		background: var(--accent);
		color: var(--surface);
		border-color: var(--accent);
	}

	.add-note {
		display: block;
		margin-bottom: 8px;
		height: 34px;
		padding: 0 16px;
		background: var(--note-fill);
		border: var(--border-width) solid var(--note-edge);
		color: var(--note-ink);
		font-size: 13px;
		cursor: pointer;
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

	.path-box {
		height: 12px;
		background: var(--path-fill);
		border: 1.5px dashed var(--node-path);
	}

	.precedes {
		height: 2px;
		background: var(--edge-precedes);
	}

	.related {
		border-top: 2px dotted var(--ink-2);
	}

	.annotation {
		border-top: 2px dashed var(--note-edge);
	}

	@media (max-width: 700px) {
		.hint {
			display: none;
		}
	}
</style>
