import { describe, expect, it } from 'vitest';
import { DEFAULT_EDGE_STYLES, arrowPath, edgeStrokeProps, edgeStyleKey } from './edgeStyle';

describe('edgeStyleKey', () => {
	it('splits requires by whether the requirement is met', () => {
		expect(edgeStyleKey('requires', true)).toBe('requires_unmet');
		expect(edgeStyleKey('requires', false)).toBe('requires_met');
	});

	it('maps the soft kinds one to one and leaves part_of unstyled', () => {
		expect(edgeStyleKey('related', false)).toBe('related');
		expect(edgeStyleKey('precedes', true)).toBe('precedes');
		expect(edgeStyleKey('part_of', false)).toBeNull();
	});
});

describe('edgeStrokeProps', () => {
	it('turns line and width presets into stroke attributes', () => {
		expect(edgeStrokeProps(DEFAULT_EDGE_STYLES.requires_met)).toEqual({
			stroke: null,
			width: 2,
			dasharray: 'none',
			linecap: 'butt'
		});
		const dotted = edgeStrokeProps({
			color: '#00ff88',
			line: 'dotted',
			width: 'thick',
			arrow: false
		});
		expect(dotted).toMatchObject({ stroke: '#00ff88', width: 3, linecap: 'round' });
		expect(dotted.dasharray).not.toBe('none');
	});

	it('defaults: related dashed without arrow, precedes dashed with arrow, requires solid', () => {
		expect(DEFAULT_EDGE_STYLES.related).toMatchObject({ line: 'dashed', arrow: false });
		expect(DEFAULT_EDGE_STYLES.precedes).toMatchObject({ line: 'dashed', arrow: true });
		expect(DEFAULT_EDGE_STYLES.requires_unmet.line).toBe('solid');
	});
});

describe('arrowPath', () => {
	it('points right when entering a left handle', () => {
		expect(arrowPath(100, 50, 'left')).toBe('M 100 50 L 91 45 L 91 55 Z');
	});

	it('points down when entering a top handle (related edges)', () => {
		expect(arrowPath(100, 50, 'top')).toBe('M 100 50 L 95 41 L 105 41 Z');
	});
});
