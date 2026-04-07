<template>
  <div class="gauge-wrapper">
    <svg
      class="gauge"
      :width="size"
      :height="size"
      viewBox="0 0 100 100"
    >
      <!-- Background circle -->
      <circle
        class="gauge-bg"
        cx="50"
        cy="50"
        r="45"
      />

      <!-- Progress circle -->
      <circle
        class="gauge-progress"
        cx="50"
        cy="50"
        r="45"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="dashOffset"
      />
    </svg>

    <!-- Center content -->
    <div class="gauge-content">
      <div class="percentage">{{ percentage }}%</div>
      <div class="label">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  percentage: {
    type: Number,
    default: 0,
  },
  size: {
    type: Number,
    default: 160,
  },
  strokeWidth: {
    type: Number,
    default: 6,
  },
})

const radius = 45
const circumference = 2 * Math.PI * radius

const dashOffset = computed(() => {
  const progress = Math.min(Math.max(props.percentage, 0), 100)

  // Shift so 100% ends around 5 o'clock (~150deg instead of full 360 start)
  const adjustedProgress = progress / 100

  return circumference * (1 - adjustedProgress)
})
</script>

<style scoped>
.gauge-wrapper {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.gauge {
  transform: rotate(140deg); /* start angle so 100% ends ~5 o'clock */
}

.gauge-bg {
  fill: none;
  stroke: #e6ecf5;
  stroke-width: 6;
}

.gauge-progress {
  fill: none;
  stroke: #003580;
  stroke-width: 6;
  stroke-linecap: round;

  transition: stroke-dashoffset 0.4s ease;
}

/* Center content */
.gauge-content {
  position: absolute;
  text-align: center;
}

.percentage {
  font-size: 22px;
  font-weight: 600;
  color: #003580;
}

.label {
  font-size: 12px;
  color: #6b7a90;
  margin-top: 4px;
}
</style>