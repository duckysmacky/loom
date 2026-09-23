<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { logout, session } from '$lib/stores/session.svelte';
	import { overlays } from '$lib/stores/ui.svelte';

	let { title }: { title: string } = $props();

	let menuOpen = $state(false);
	const initial = $derived(session.user?.email.charAt(0).toUpperCase() ?? '?');
</script>

<header class="topbar">
	<h1 class="title">{title}</h1>

	<button type="button" class="search" onclick={() => (overlays.paletteOpen = true)}>
		<span class="lens"></span>
		<span class="placeholder">Search nodes…</span>
		<span class="key">⌘K</span>
	</button>

	<div class="actions">
		<Button variant="accent" onclick={() => (overlays.captureOpen = true)}>
			+<span class="capture-label"> Quick capture</span>
		</Button>
		<div class="account">
			<button
				type="button"
				class="avatar"
				aria-label="Account menu"
				aria-expanded={menuOpen}
				onclick={() => (menuOpen = !menuOpen)}>{initial}</button
			>
			{#if menuOpen}
				<div class="menu">
					<div class="email">{session.user?.email}</div>
					<a href="/settings" onclick={() => (menuOpen = false)}>Settings</a>
					<button type="button" onclick={logout}>Sign out</button>
				</div>
			{/if}
		</div>
	</div>
</header>

<svelte:window
	onclick={(event) => {
		if (menuOpen && !(event.target as Element).closest('.account')) menuOpen = false;
	}}
/>

<style>
	.topbar {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 13px 22px;
		background: var(--surface);
		border-bottom: var(--border-width-hair) solid var(--line);
		position: sticky;
		top: 0;
		z-index: 20;
	}

	.title {
		margin: 0;
		font: 700 16px/1 var(--font-display);
	}

	.search {
		flex: 1;
		max-width: 320px;
		margin-left: 8px;
		display: flex;
		align-items: center;
		gap: 8px;
		background: var(--surface-2);
		border: var(--border-width-hair) solid var(--line);
		padding: 8px 11px;
		text-align: left;
	}

	.lens {
		width: 11px;
		height: 11px;
		border: 1.5px solid var(--ink-2);
		border-radius: 50%;
		flex: none;
	}

	.placeholder {
		font: 500 13px/1 var(--font-display);
		color: var(--ink-2);
	}

	.key {
		margin-left: auto;
		font: 600 10.5px/1 var(--font-mono);
		color: var(--ink-2);
		background: var(--chip);
		padding: 4px 6px;
	}

	.actions {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.account {
		position: relative;
	}

	.avatar {
		width: 30px;
		height: 30px;
		border-radius: 50%;
		border: none;
		background: var(--chip);
		font: 700 12px/1 var(--font-mono);
		color: var(--ink-3);
	}

	.menu {
		position: absolute;
		right: 0;
		top: calc(100% + 8px);
		min-width: 200px;
		background: var(--surface);
		border: var(--border-width) solid var(--frame);
		display: flex;
		flex-direction: column;
	}

	.menu > * {
		padding: 10px 14px;
		font: 600 12.5px/1.2 var(--font-display);
		text-align: left;
		color: var(--ink);
		background: none;
		border: none;
	}

	.menu > a:hover,
	.menu > button:hover {
		background: var(--surface-2);
	}

	@media (max-width: 700px) {
		.topbar {
			padding: 10px 14px;
			gap: 10px;
			position: static;
		}

		.search {
			margin-left: 0;
			min-width: 0;
		}

		.placeholder,
		.key {
			display: none;
		}

		.search {
			flex: 0 0 auto;
			padding: 9px 10px;
		}

		.capture-label {
			display: none;
		}
	}

	.email {
		font: 400 10.5px/1.3 var(--font-mono);
		color: var(--ink-2);
		border-bottom: var(--border-width-hair) solid var(--line);
		word-break: break-all;
	}
</style>
