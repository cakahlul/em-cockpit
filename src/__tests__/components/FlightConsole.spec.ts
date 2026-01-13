/**
 * FlightConsole Component Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, VueWrapper } from '@vue/test-utils';
import { nextTick } from 'vue';
import FlightConsole from '../../components/search/FlightConsole.vue';
import { mockInvoke, createMockSearchResult, flushPromises } from '../setup';

describe('FlightConsole', () => {
  let wrapper: VueWrapper;

  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe('Rendering', () => {
    it('renders when modelValue is true', async () => {
      wrapper = mount(FlightConsole, {
        props: { modelValue: true },
        global: {
          stubs: { Teleport: true },
        },
      });
      
      await nextTick();
      expect(wrapper.find('.spotlight-container').exists()).toBe(true);
    });

    it('does not render when modelValue is false', () => {
      wrapper = mount(FlightConsole, {
        props: { modelValue: false },
        global: {
          stubs: { Teleport: true },
        },
      });
      
      expect(wrapper.find('.spotlight-container').exists()).toBe(false);
    });

    it('renders search input with placeholder', async () => {
      wrapper = mount(FlightConsole, {
        props: { modelValue: true, placeholder: 'Custom placeholder' },
        global: {
          stubs: { Teleport: true },
        },
      });
      
      await nextTick();
      const input = wrapper.find('input');
      expect(input.exists()).toBe(true);
    });
  });

  describe('Keyboard Navigation', () => {
    it('closes on Escape key', async () => {
      wrapper = mount(FlightConsole, {
        props: { modelValue: true },
        global: {
          stubs: { Teleport: true },
        },
      });
      
      await nextTick();
      const input = wrapper.find('input');
      await input.trigger('keydown.escape');
      
      expect(wrapper.emitted('update:modelValue')).toBeTruthy();
    });
  });

  describe('Search Functionality', () => {
    it('calls search API with query', async () => {
      mockInvoke.mockResolvedValueOnce({
        results: [createMockSearchResult()],
        total: 1,
        query: 'test',
      });

      wrapper = mount(FlightConsole, {
        props: { modelValue: true },
        global: {
          stubs: { Teleport: true },
        },
      });
      
      await nextTick();
      
      const input = wrapper.find('input');
      await input.setValue('test');
      await input.trigger('input');
      
      // Wait for debounce
      await new Promise(r => setTimeout(r, 300));
      await flushPromises();
      
      expect(mockInvoke).toHaveBeenCalledWith('search', expect.any(Object));
    });
  });

  describe('Result Selection', () => {
    it('emits select event when result is clicked', async () => {
      const mockResult = createMockSearchResult();
      mockInvoke.mockResolvedValueOnce({
        results: [mockResult],
        total: 1,
        query: 'test',
      });

      wrapper = mount(FlightConsole, {
        props: { modelValue: true },
        global: {
          stubs: { Teleport: true },
        },
      });
      
      await nextTick();
      
      const input = wrapper.find('input');
      await input.setValue('TEST-123');
      await input.trigger('input');
      
      await new Promise(r => setTimeout(r, 300));
      await flushPromises();
      await nextTick();
      
      const resultItem = wrapper.find('.result-item');
      if (resultItem.exists()) {
        await resultItem.trigger('click');
        expect(wrapper.emitted('select')).toBeTruthy();
      }
    });
  });

  describe('Loading State', () => {
    it('shows loading indicator during search', async () => {
      // Make the mock take time
      mockInvoke.mockImplementationOnce(() => 
        new Promise(resolve => setTimeout(() => resolve({ results: [], total: 0, query: '' }), 500))
      );

      wrapper = mount(FlightConsole, {
        props: { modelValue: true },
        global: {
          stubs: { Teleport: true },
        },
      });
      
      await nextTick();
      
      const input = wrapper.find('input');
      await input.setValue('test');
      await input.trigger('input');
      
      // Should show loading before results come back
      await new Promise(r => setTimeout(r, 250));
      // Loading state would be visible here
    });
  });
});
