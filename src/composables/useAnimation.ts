/**
 * Animation Composable
 * 
 * Provides Vue-friendly animation utilities and hooks
 * for managing smooth transitions and micro-interactions.
 */

import { ref, computed, onMounted, onUnmounted, watch, Ref } from 'vue';

// Animation duration tokens (match CSS)
export const AnimationDurations = {
  instant: 50,
  fast: 150,
  normal: 250,
  slow: 350,
  slower: 500,
} as const;

/**
 * Hook for managing enter/exit animations
 */
export function useAnimation(isVisible: Ref<boolean>, duration = AnimationDurations.normal) {
  const isAnimating = ref(false);
  const shouldRender = ref(isVisible.value);

  watch(isVisible, (newValue) => {
    if (newValue) {
      // Entering - render immediately, then animate
      shouldRender.value = true;
      isAnimating.value = true;
      setTimeout(() => {
        isAnimating.value = false;
      }, duration);
    } else {
      // Exiting - animate, then stop rendering
      isAnimating.value = true;
      setTimeout(() => {
        shouldRender.value = false;
        isAnimating.value = false;
      }, duration);
    }
  });

  return {
    isAnimating,
    shouldRender,
    animationClass: computed(() => {
      if (!isVisible.value && isAnimating.value) return 'animate-exit';
      if (isVisible.value && isAnimating.value) return 'animate-enter';
      return '';
    }),
  };
}

/**
 * Hook for staggered list animations
 */
export function useStaggeredList<T>(
  items: Ref<T[]>,
  staggerDelay = 50
) {
  const visibleCount = ref(0);

  watch(items, (newItems) => {
    // Reset and animate items one by one
    visibleCount.value = 0;
    const interval = setInterval(() => {
      if (visibleCount.value < newItems.length) {
        visibleCount.value++;
      } else {
        clearInterval(interval);
      }
    }, staggerDelay);
  }, { immediate: true });

  return {
    visibleCount,
    getItemProps: (index: number) => ({
      style: {
        opacity: index < visibleCount.value ? 1 : 0,
        transform: index < visibleCount.value ? 'translateX(0)' : 'translateX(-10px)',
        transition: `opacity ${AnimationDurations.normal}ms ease, transform ${AnimationDurations.normal}ms ease`,
      },
    }),
  };
}

/**
 * Hook for counting number animation
 */
export function useCountAnimation(
  targetValue: Ref<number>,
  duration = 500
) {
  const currentValue = ref(0);
  const isAnimating = ref(false);

  watch(targetValue, (newValue, oldValue = 0) => {
    const startValue = oldValue;
    const difference = newValue - startValue;
    const startTime = performance.now();
    isAnimating.value = true;

    function animate(currentTime: number) {
      const elapsed = currentTime - startTime;
      const progress = Math.min(elapsed / duration, 1);
      
      // Ease out cubic
      const eased = 1 - Math.pow(1 - progress, 3);
      currentValue.value = Math.round(startValue + difference * eased);

      if (progress < 1) {
        requestAnimationFrame(animate);
      } else {
        currentValue.value = newValue;
        isAnimating.value = false;
      }
    }

    requestAnimationFrame(animate);
  }, { immediate: true });

  return {
    displayValue: currentValue,
    isAnimating,
  };
}

/**
 * Hook for pulsing animation
 */
export function usePulse(intervalMs = 2000) {
  const isPulsing = ref(false);
  let timer: ReturnType<typeof setInterval> | null = null;

  onMounted(() => {
    timer = setInterval(() => {
      isPulsing.value = true;
      setTimeout(() => {
        isPulsing.value = false;
      }, intervalMs / 2);
    }, intervalMs);
  });

  onUnmounted(() => {
    if (timer) {
      clearInterval(timer);
    }
  });

  return { isPulsing };
}

/**
 * Hook for detecting reduced motion preference
 */
export function useReducedMotion() {
  const prefersReducedMotion = ref(false);

  onMounted(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    prefersReducedMotion.value = mediaQuery.matches;

    const handler = (e: MediaQueryListEvent) => {
      prefersReducedMotion.value = e.matches;
    };

    mediaQuery.addEventListener('change', handler);
    
    onUnmounted(() => {
      mediaQuery.removeEventListener('change', handler);
    });
  });

  return { prefersReducedMotion };
}

/**
 * Hook for intersection-based animations
 */
export function useScrollAnimation(
  threshold = 0.1
) {
  const elementRef = ref<HTMLElement | null>(null);
  const isVisible = ref(false);
  const hasAnimated = ref(false);

  onMounted(() => {
    if (!elementRef.value) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.value) {
          isVisible.value = true;
          hasAnimated.value = true;
          observer.disconnect();
        }
      },
      { threshold }
    );

    observer.observe(elementRef.value);

    onUnmounted(() => {
      observer.disconnect();
    });
  });

  return {
    elementRef,
    isVisible,
    animationClass: computed(() => isVisible.value ? 'animate-visible' : 'animate-hidden'),
  };
}

/**
 * Hook for typewriter text animation
 */
export function useTypewriter(
  text: Ref<string>,
  speed = 50
) {
  const displayedText = ref('');
  const isTyping = ref(false);
  const cursorVisible = ref(true);

  let cursorInterval: ReturnType<typeof setInterval> | null = null;

  watch(text, async (newText) => {
    displayedText.value = '';
    isTyping.value = true;

    for (let i = 0; i <= newText.length; i++) {
      displayedText.value = newText.slice(0, i);
      await new Promise(resolve => setTimeout(resolve, speed));
    }

    isTyping.value = false;
  }, { immediate: true });

  onMounted(() => {
    cursorInterval = setInterval(() => {
      cursorVisible.value = !cursorVisible.value;
    }, 500);
  });

  onUnmounted(() => {
    if (cursorInterval) {
      clearInterval(cursorInterval);
    }
  });

  return {
    displayedText,
    isTyping,
    cursorVisible,
    textWithCursor: computed(() => 
      displayedText.value + (cursorVisible.value ? '|' : '')
    ),
  };
}

/**
 * Debounced animation trigger
 */
export function useDebouncedAnimation(delay = 250) {
  const isActive = ref(false);
  let timeout: ReturnType<typeof setTimeout> | null = null;

  function trigger() {
    isActive.value = true;
    
    if (timeout) {
      clearTimeout(timeout);
    }
    
    timeout = setTimeout(() => {
      isActive.value = false;
    }, delay);
  }

  onUnmounted(() => {
    if (timeout) {
      clearTimeout(timeout);
    }
  });

  return {
    isActive,
    trigger,
  };
}

/**
 * Spring physics animation
 */
export function useSpring(
  target: Ref<number>,
  stiffness = 100,
  damping = 10
) {
  const current = ref(target.value);
  const velocity = ref(0);
  let animationFrame: number | null = null;

  function animate() {
    const displacement = target.value - current.value;
    const springForce = displacement * stiffness * 0.001;
    const dampingForce = velocity.value * damping * 0.001;
    
    velocity.value += springForce - dampingForce;
    current.value += velocity.value;

    // Stop if close enough
    if (Math.abs(displacement) < 0.01 && Math.abs(velocity.value) < 0.01) {
      current.value = target.value;
      animationFrame = null;
      return;
    }

    animationFrame = requestAnimationFrame(animate);
  }

  watch(target, () => {
    if (!animationFrame) {
      animationFrame = requestAnimationFrame(animate);
    }
  });

  onUnmounted(() => {
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  });

  return { current };
}
