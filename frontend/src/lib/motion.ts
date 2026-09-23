/**
 * Motion durations for Svelte transitions. CSS animations are switched off
 * by a `prefers-reduced-motion` rule in global.css; JS-driven transitions
 * need the same check, so every duration goes through `ms()`.
 */
const reduceMotion =
	typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

export const ms = (duration: number) => (reduceMotion ? 0 : duration);
