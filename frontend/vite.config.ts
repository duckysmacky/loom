import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';

export default defineConfig({
	plugins: [
		// UI messages (`messages/<locale>.json`) compiled to `$lib/paraglide`.
		// Locale: the user's override in localStorage, else the browser's
		// language, else English.
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide',
			strategy: ['localStorage', 'preferredLanguage', 'baseLocale'],
			localStorageKey: 'loom.locale'
		}),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// Plain static SPA served by its own nginx container - every
			// unknown path falls back to index.html and client-side routing
			// takes over.
			adapter: adapter({ fallback: 'index.html' })
		})
	],
	server: {
		// Dev only: the backend runs separately on :8080 (`cargo run`).
		// `changeOrigin: false` keeps the Host header, which the MCP
		// endpoint checks against PUBLIC_URL.
		proxy: {
			'/api': 'http://localhost:8080',
			'/mcp': { target: 'http://localhost:8080', changeOrigin: false },
			'/.well-known/oauth-': 'http://localhost:8080'
		}
	},
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: './vite.config.ts',
				test: {
					name: 'client',
					browser: {
						enabled: true,
						provider: playwright(),
						instances: [{ browser: 'chromium', headless: true }]
					},
					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					exclude: ['src/lib/server/**']
				}
			},

			{
				extends: './vite.config.ts',
				test: {
					name: 'server',
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
