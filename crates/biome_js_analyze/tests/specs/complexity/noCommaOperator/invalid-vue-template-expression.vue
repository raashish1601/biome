<!-- should generate diagnostics -->
<script setup>
const value = 1;
</script>

<template>
  <div>{{ (console.log("side effect"), value) }}</div>
</template>
