<script setup lang="ts">
import { computed } from 'vue'
import type { Node, NoteType } from '@/bindings'

const props = withDefaults(defineProps<{
  node: Node
  noteType: NoteType | null
  cardSlot?: 'front' | 'back' | 'split'
}>(), {
  cardSlot: 'split',
})

const renderedFront = computed(() => props.node.note_fields.RenderedFrontHtml ?? '')
const renderedBack = computed(() => props.node.note_fields.RenderedBackHtml ?? '')
const renderedCss = computed(() => props.node.note_fields.RenderedCss ?? '')
const renderedThemeMode = computed(() => props.node.note_fields.RenderedThemeMode ?? 'clean')
const renderedMediaMode = computed(() => props.node.note_fields.RenderedMediaMode ?? 'omitted')

function sanitizeFragment(fragment: string): string {
  if (!fragment.trim()) return ''
  return fragment
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gis, '')
    .replace(/\son[a-z-]+\s*=\s*"[^"]*"/gis, '')
    .replace(/\son[a-z-]+\s*=\s*'[^']*'/gis, '')
    .replace(/\son[a-z-]+\s*=\s*[^\s>]+/gis, '')
    .replace(/javascript:/gis, '#blocked-script:')
}

function buildSrcDoc(fragment: string): string {
  const csp = "default-src 'none'; img-src data: http: https: blob:; media-src data: http: https: blob:; style-src 'unsafe-inline'; font-src data: http: https:; connect-src 'none'; script-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'"
  const shellCss = `
    :root {
      color-scheme: dark;
      --card-bg: #0f1321;
      --card-panel: rgba(255,255,255,0.03);
      --card-border: rgba(255,255,255,0.08);
      --card-text: #dce4ff;
      --card-muted: #8f97b2;
      font-family: Inter, ui-sans-serif, system-ui, sans-serif;
    }
    * { box-sizing: border-box; }
    html, body { margin: 0; padding: 0; background: var(--card-bg); color: var(--card-text); }
    body { padding: 18px; line-height: 1.6; }
    img { max-width: 100%; height: auto; display: block; border-radius: 12px; }
    audio, video { width: 100%; max-width: 420px; }
    section { margin: 0 0 18px; }
    h1, h2, h3, h4 { margin: 0 0 10px; }
    p, ul, ol { margin: 0 0 10px; }
    .concept-card-clean section {
      padding: 14px 16px;
      border-radius: 14px;
      background: var(--card-panel);
      border: 1px solid var(--card-border);
    }
    .concept-card-empty {
      padding: 18px;
      border-radius: 14px;
      border: 1px dashed var(--card-border);
      color: var(--card-muted);
    }
  `
  const modeClass = renderedThemeMode.value === 'source' ? 'concept-card-source' : 'concept-card-clean'
  const safeFragment = sanitizeFragment(fragment)
  const body = safeFragment.trim() || '<div class="concept-card-empty">No rendered card content available.</div>'
  const sourceCss = renderedThemeMode.value === 'source' && renderedCss.value.trim() ? renderedCss.value : ''
  return `<!doctype html><html><head><meta charset="utf-8" /><meta name="viewport" content="width=device-width, initial-scale=1" /><meta http-equiv="Content-Security-Policy" content="${csp}" /><style>${shellCss}</style>${sourceCss ? `<style>${sourceCss}</style>` : ''}</head><body><div class="${modeClass}" data-media-mode="${renderedMediaMode.value}">${body}</div></body></html>`
}

const frontDoc = computed(() => buildSrcDoc(renderedFront.value))
const backDoc = computed(() => buildSrcDoc(renderedBack.value))
</script>

<template>
  <div class="anki-card-page" :class="`slot-${cardSlot}`">
    <article v-if="cardSlot === 'front' || cardSlot === 'split'" class="card-frame-wrap">
      <div class="card-frame-label">Front</div>
      <iframe class="card-frame" :srcdoc="frontDoc" sandbox="allow-same-origin" referrerpolicy="no-referrer" />
    </article>

    <article v-if="cardSlot === 'back' || cardSlot === 'split'" class="card-frame-wrap">
      <div class="card-frame-label">Back</div>
      <iframe class="card-frame" :srcdoc="backDoc" sandbox="allow-same-origin" referrerpolicy="no-referrer" />
    </article>
  </div>
</template>

<style scoped>
.anki-card-page {
  display: grid;
  gap: 18px;
  width: 100%;
}

.anki-card-page.slot-split {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.card-frame-wrap {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.card-frame-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.card-frame {
  width: 100%;
  min-height: 620px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 18px;
  background: #0f1321;
}

@media (max-width: 1100px) {
  .anki-card-page.slot-split {
    grid-template-columns: minmax(0, 1fr);
  }

  .card-frame {
    min-height: 420px;
  }
}
</style>
