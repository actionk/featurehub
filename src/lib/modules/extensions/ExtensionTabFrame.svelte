<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { Feature } from "../../api/types";

  let { featureId, feature, componentPath }: {
    featureId: string;
    feature: Feature;
    componentPath: string;
    [key: string]: unknown;
  } = $props();

  let iframeEl = $state<HTMLIFrameElement | null>(null);

  // convertFileSrc handles platform differences (drive letters on Windows,
  // proper scheme registration via Tauri's asset protocol).
  const assetUrl = $derived(convertFileSrc(componentPath, "asset"));

  function handleLoad() {
    if (!iframeEl?.contentWindow) return;
    iframeEl.contentWindow.postMessage(
      {
        type: "fh:init",
        featureId,
        feature,
      },
      "*"
    );
  }

  function handleMessage(event: MessageEvent) {
    if (event.data?.type !== "fh:invoke") return;
    const { command, params, requestId } = event.data;
    if (!command || !requestId) return;

    import("@tauri-apps/api/core")
      .then(({ invoke }) => invoke(command, params ?? {}))
      .then((data) => {
        iframeEl?.contentWindow?.postMessage(
          { type: "fh:invoke-result", requestId, ok: true, data },
          "*"
        );
      })
      .catch((err) => {
        iframeEl?.contentWindow?.postMessage(
          {
            type: "fh:invoke-result",
            requestId,
            ok: false,
            error: String(err),
          },
          "*"
        );
      });
  }
</script>

<svelte:window onmessage={handleMessage} />

<iframe
  bind:this={iframeEl}
  src={assetUrl}
  onload={handleLoad}
  title="Extension tab"
  style="width: 100%; height: 100%; border: none; background: transparent;"
  sandbox="allow-scripts allow-same-origin allow-top-navigation-to-custom-protocols allow-popups"
></iframe>
