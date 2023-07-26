<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import prettyMS from 'pretty-ms'

const recordMax = 3 * 60 * 60
const recordSeconds = ref(300)

const recordStep = computed(() => {
  return 60
})

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// greetMsg.value = await invoke("greet", { name: name.value });

const submitting = ref(false)
const updateConfig = async () => {
  submitting.value = true
  await nextTick()
  await invoke('update_config', { recordSecondsStr: `${recordSeconds.value}` })
  await nextTick()
  submitting.value = false
}

</script>

<template>
  <div class="container">
    <div class="field">
      <label class="label">Rewind Duration</label>
      <div class="control">
        <div class="slider-container">
          <input class="slider is-fullwidth" :step="recordStep" min="0" :max="recordMax" type="range" v-model="recordSeconds">
          <span>{{ prettyMS(recordSeconds * 1000, { verbose: true }) }}</span>
        </div>
      </div>
    </div>

    <div class="submit-container">
      <button class="button is-link" :disabled="submitting" @click="updateConfig">Apply</button>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.field .label {
  font-weight: bold;
}

.container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
.slider-container {
  display: inline-flex;
  flex-direction: row;
  flex: 1;
  input, input:before, input:after, input:focus {
    outline: none !important;
    border: none !important;
  }
  input + span {
    padding-left: 12px;
  }
}
.submit-container {
  margin-top: 4em;
}
</style>