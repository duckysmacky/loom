<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Variant = 'primary' | 'secondary' | 'poke' | 'accent' | 'quiet';

	let {
		variant = 'secondary',
		type = 'button',
		children,
		...rest
	}: HTMLButtonAttributes & { variant?: Variant; children: Snippet } = $props();
</script>

<!-- Loom Button: primary (solid ink, one per view), secondary (ink outline),
poke (accent text on a hairline - the low-emphasis nudge), accent (solid
accent, top-bar calls to action), quiet (hairline, ink-3 text). -->
<button {type} class="button {variant}" {...rest}>{@render children()}</button>

<style>
	.button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		font: 700 11px/1 var(--font-mono);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		padding: 10px 16px;
		white-space: nowrap;
		border: var(--border-width) solid var(--ink);
		background: var(--surface);
		color: var(--ink);
		transition:
			opacity var(--fast) var(--ease),
			transform var(--fast) var(--ease),
			border-color var(--fast) var(--ease);
	}

	.button:not(:disabled):hover {
		opacity: 0.85;
	}

	.button:not(:disabled):active {
		transform: translateY(1px);
	}

	.primary {
		background: var(--ink);
		color: var(--on-ink);
	}

	.poke,
	.quiet {
		border-width: var(--border-width-hair);
		border-color: var(--line);
		padding: 7px 10px;
		font-size: 10.5px;
	}

	.poke {
		color: var(--accent);
	}

	.quiet {
		color: var(--ink-3);
	}

	/* Top-bar style call to action: display face, sentence case. */
	.accent {
		font: 700 12.5px/1 var(--font-display);
		letter-spacing: 0;
		text-transform: none;
		padding: 10px 14px;
		background: var(--accent);
		border-color: var(--accent);
		color: var(--on-accent);
	}
</style>
