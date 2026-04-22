<script lang="ts">
  import type { Skill, FeatureSkill } from "../../api/tauri";
  import { getFeatureSkills, setFeatureSkill } from "../../api/tauri";
  import { getCachedSettings } from "../../stores/settings.svelte";

  let {
    featureId,
  }: {
    featureId: string;
  } = $props();

  let allSkills = $state<Skill[]>([]);
  let featureOverrides = $state<FeatureSkill[]>([]);
  let loading = $state(true);

  let enabledCount = $derived(allSkills.filter(isSkillEnabled).length);

  function isSkillEnabled(skill: Skill): boolean {
    const override = featureOverrides.find((o) => o.skill_id === skill.id);
    if (override) return override.enabled;
    return skill.default_enabled;
  }

  async function toggleSkill(skill: Skill) {
    const current = isSkillEnabled(skill);
    await setFeatureSkill(featureId, skill.id, !current);
    const existing = featureOverrides.findIndex((o) => o.skill_id === skill.id);
    if (existing >= 0) {
      featureOverrides[existing] = { skill_id: skill.id, enabled: !current };
    } else {
      featureOverrides = [...featureOverrides, { skill_id: skill.id, enabled: !current }];
    }
  }

  $effect(() => {
    loadData();
  });

  async function loadData() {
    loading = true;
    try {
      const [settings, overrides] = await Promise.all([
        getCachedSettings(),
        getFeatureSkills(featureId),
      ]);
      allSkills = settings.skills ?? [];
      featureOverrides = overrides;
    } catch (e) {
      console.error("Failed to load skills:", e);
    } finally {
      loading = false;
    }
  }
</script>

{#if !loading && allSkills.length > 0}
  <div class="skills-section">
    <div class="skills-header">
      <svg class="skills-icon" width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M7.5 1.75a.75.75 0 0 1 .75.75v.5h3.25a.75.75 0 0 1 0 1.5H8.25v1.25a.75.75 0 0 1-1.5 0V4.5H3.5a.75.75 0 0 1 0-1.5h3.25v-.5a.75.75 0 0 1 .75-.75ZM3.5 8.75a.75.75 0 0 0 0 1.5h3.25v1.25a.75.75 0 0 0 1.5 0V10.25H11.5a.75.75 0 0 0 0-1.5H8.25V7.5a.75.75 0 0 0-1.5 0v1.25H3.5Z"/></svg>
      <span class="skills-title">Skills</span>
      <span class="skills-count">{enabledCount}/{allSkills.length}</span>
    </div>
    <div class="skills-blocks">
      {#each allSkills as skill (skill.id)}
        {@const enabled = isSkillEnabled(skill)}
        <button
          class="skill-block"
          class:skill-block--on={enabled}
          onclick={() => toggleSkill(skill)}
          title={skill.name}
        >
          <span class="skill-dot" class:skill-dot--on={enabled}></span>
          <span class="skill-name">{skill.name}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .skills-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 14px;
  }

  .skills-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .skills-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .skills-title {
    font-size: 11.5px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .skills-count {
    font-size: 10.5px;
    font-weight: 500;
    color: var(--text-muted);
    font-family: var(--font-mono);
    opacity: 0.7;
  }

  .skills-blocks {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .skill-block {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 5px 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-card);
    color: var(--text-muted);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .skill-block:hover {
    background: var(--bg-hover);
    border-color: color-mix(in srgb, var(--text-muted) 30%, var(--border));
  }
  .skill-block--on {
    color: var(--text-primary);
    border-color: color-mix(in srgb, var(--purple) 30%, var(--border));
    background: color-mix(in srgb, var(--purple) 5%, var(--bg-card));
  }
  .skill-block--on:hover {
    border-color: color-mix(in srgb, var(--purple) 50%, var(--border));
    background: color-mix(in srgb, var(--purple) 8%, var(--bg-card));
  }

  .skill-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--text-muted);
    opacity: 0.3;
    transition: all 0.15s;
  }
  .skill-dot--on {
    background: var(--purple);
    opacity: 1;
    box-shadow: 0 0 4px color-mix(in srgb, var(--purple) 50%, transparent);
  }

  .skill-name {
    font-size: 12px;
    font-weight: 500;
  }
</style>
