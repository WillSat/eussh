<script setup>
import { useI18n } from '@/composables/useI18n'
const { t } = useI18n()
</script>

<template>
  <div class="h-full w-full flex items-center justify-center bg-[var(--color-bg-primary)] select-none">
    <div class="text-center">
      <h1 class="text-3xl font-light tracking-tight text-[var(--color-text-primary)] mb-3">{{ t('welcome.heading') }}</h1>
      <p class="text-sm text-[var(--color-text-secondary)]">{{ t('welcome.subtitle') }}</p>
      <p class="text-xs text-[var(--color-text-tertiary)] mt-6">{{ t('welcome.hint') }}</p>
    </div>
  </div>
</template>
