<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import './settings.css';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { getLocale, locales, localStorageKey, setLocale } from '$lib/paraglide/runtime';
	import { prefs, savePrefs } from '$lib/stores/prefs.svelte';

	// Per-browser override of the browser-derived language (Paraglide keeps it
	// in localStorage); 'system' clears it. Both reload the page.
	const languageOptions = [
		{ value: 'system', label: m.appearance_system() },
		...locales.map((locale) => {
			const name = new Intl.DisplayNames([locale], { type: 'language' }).of(locale) ?? locale;
			return { value: locale, label: name[0].toLocaleUpperCase(locale) + name.slice(1) };
		})
	];
	const language = (() => {
		try {
			return localStorage.getItem(localStorageKey) ? getLocale() : 'system';
		} catch {
			return 'system';
		}
	})();

	function chooseLanguage(value: string) {
		if (value !== 'system') return setLocale(value as (typeof locales)[number]);
		try {
			localStorage.removeItem(localStorageKey);
		} catch {
			// Storage blocked: nothing was saved to clear.
		}
		location.reload();
	}
</script>

<section class="settings-section">
	<h2>{m.settings_appearance()}</h2>
	<p class="explain">{m.appearance_explain()}</p>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.appearance_theme()}</span>
			<span class="row-help">{m.appearance_theme_help()}</span>
		</span>
		<SegmentedControl
			label={m.appearance_theme()}
			value={prefs.theme}
			onchange={(theme) => {
				prefs.theme = theme;
				savePrefs();
			}}
			options={[
				{ value: 'system', label: m.appearance_system() },
				{ value: 'light', label: m.appearance_light() },
				{ value: 'dark', label: m.appearance_dark() }
			]}
		/>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.appearance_density()}</span>
			<span class="row-help">{m.appearance_density_help()}</span>
		</span>
		<SegmentedControl
			label={m.appearance_density()}
			value={prefs.density}
			onchange={(density) => {
				prefs.density = density;
				savePrefs();
			}}
			options={[
				{ value: 'comfortable', label: m.appearance_comfortable() },
				{ value: 'compact', label: m.appearance_compact() }
			]}
		/>
	</div>
	<div class="settings-row">
		<span>
			<span class="row-label">{m.appearance_icon_style()}</span>
			<span class="row-help">{m.appearance_icon_style_help()}</span>
		</span>
		<SegmentedControl
			label={m.appearance_icon_style()}
			value={prefs.iconStyle}
			onchange={(iconStyle) => {
				prefs.iconStyle = iconStyle;
				savePrefs();
			}}
			options={[
				{ value: 'icons', label: m.appearance_icons() },
				{ value: 'glyphs', label: m.appearance_glyphs() }
			]}
		/>
	</div>
	<!-- Hidden until there's more than one language to pick. -->
	{#if locales.length > 1}
		<div class="settings-row">
			<span>
				<span class="row-label">{m.appearance_language()}</span>
				<span class="row-help">{m.appearance_language_help()}</span>
			</span>
			<SegmentedControl
				label={m.appearance_language()}
				value={language}
				onchange={chooseLanguage}
				options={languageOptions}
			/>
		</div>
	{/if}
</section>
