<script setup lang="ts">
useSeo({
  title: 'Why do you think I need an admin panel?',
  description: 'There is no admin panel. Keep things simple.',
  noindex: true,
})
</script>

<template>
  <div class="lh-text lh-gap">
    <EmptyState
      as="h1"
      eyebrow="no such panel"
      heading="Why do you think I need an admin panel?"
      detail="Everything administrative is either a static file in a private repo or a bash script running on the VM. Keep things simple."
      :action="{ label: 'Back to the shelf', to: '/' }"
    />
  </div>
</template>
