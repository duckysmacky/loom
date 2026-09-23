<script lang="ts">
	import { page } from '$app/state';

	const destinations = [
		{ href: '/dashboard', label: 'Dashboard' },
		{
			href: '/board',
			label: 'Board',
			children: [
				{ href: '/board/organized', label: 'Organized' },
				{ href: '/board/canvas', label: 'Canvas' },
				{ href: '/board/timeline', label: 'Timeline' }
			]
		},
		{ href: '/nodes', label: 'Nodes' },
		{ href: '/settings', label: 'Settings' }
	];

	const isActive = (href: string) => page.url.pathname.startsWith(href);
</script>

<nav class="sidebar" aria-label="Main">
	<a class="brand" href="/dashboard">
		<span class="mark"></span>
		<span class="wordmark">LOOM</span>
	</a>

	<ul class="items">
		{#each destinations as destination (destination.href)}
			<li>
				<!-- NavItem: square marker, filled with accent when active. -->
				<a
					class="item"
					class:active={isActive(destination.href)}
					href={destination.href}
					aria-current={isActive(destination.href) ? 'page' : undefined}
				>
					<span class="marker"></span>{destination.label}
				</a>
				{#if destination.children && isActive(destination.href)}
					<ul>
						{#each destination.children as child (child.href)}
							<li>
								<a
									class="sub"
									class:active={isActive(child.href)}
									href={child.href}
									aria-current={isActive(child.href) ? 'page' : undefined}>{child.label}</a
								>
							</li>
						{/each}
					</ul>
				{/if}
			</li>
		{/each}
	</ul>

	<div class="hints">
		<span class="key">Ctrl K</span> jump to node<br />
		<span class="key">N</span> quick capture
	</div>
</nav>

<style>
	.sidebar {
		width: 210px;
		flex: none;
		background: var(--side);
		color: var(--side-ink);
		display: flex;
		flex-direction: column;
		padding: 18px 0;
		border-right: var(--border-width-hair) solid var(--side-line);
		position: sticky;
		top: 0;
		height: 100vh;
	}

	.brand {
		padding: 0 18px 20px;
		display: flex;
		align-items: center;
		gap: 9px;
	}

	.mark {
		width: 15px;
		height: 15px;
		background: var(--accent);
	}

	.wordmark {
		font: 700 17px/1 var(--font-mono);
		letter-spacing: 0.06em;
		color: var(--side-hi);
	}

	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.items {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 0 10px;
	}

	.item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 12px;
		color: var(--side-ink);
		font: 500 13.5px/1.2 var(--font-display);
	}

	.item.active {
		background: var(--side-active);
		color: var(--side-hi);
		font-weight: 700;
	}

	.marker {
		width: 8px;
		height: 8px;
		border: 1.5px solid var(--side-ink);
	}

	.active .marker {
		background: var(--accent);
		border-color: var(--accent);
	}

	.sub {
		display: block;
		padding: 7px 12px 7px 30px;
		color: var(--side-ink);
		font: 500 12.5px/1.2 var(--font-display);
	}

	.sub.active {
		color: var(--side-hi);
		font-weight: 700;
	}

	.hints {
		margin: auto 18px 0;
		border: var(--border-width-hair) dashed var(--side-line);
		padding: 10px 12px;
		font: 500 11.5px/1.6 var(--font-display);
	}

	.key {
		font: 700 11px/1 var(--font-mono);
		color: var(--side-hi);
	}

	@media (max-width: 900px) {
		.sidebar {
			width: auto;
			height: auto;
			position: static;
			flex-direction: row;
			align-items: center;
			padding: 10px 12px;
			border-right: none;
			border-bottom: var(--border-width-hair) solid var(--side-line);
			overflow-x: auto;
		}

		.brand {
			padding: 0 14px 0 4px;
		}

		.items {
			flex-direction: row;
			padding: 0;
		}

		.item {
			padding: 8px 10px;
			white-space: nowrap;
		}

		/* Board views are one click away in the board's own switcher. */
		.items ul,
		.hints {
			display: none;
		}
	}
</style>
