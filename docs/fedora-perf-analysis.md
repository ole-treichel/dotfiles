# Fedora sluggishness analysis — 2026-08-24

Machine: Lenovo laptop, 13th Gen Intel i7-1370P (14C/20T, Iris Xe iGPU only),
62GB RAM, 1x NVMe (btrfs), Fedora 42, kernel 6.19.14, GNOME 48 on Wayland,
uptime 16 days. Complaint: general sluggishness, poor "snappiness", despite
strong specs. This is a diagnostic pass only — nothing was changed.

## Findings, ranked by likely impact

### 1. Two stray `prism` debug processes running non-stop for 5 days

```
PID 978182 — 5-00:20:07 elapsed, 1199 min CPU time (~16.6% of a core, continuously)
PID 978213 — 5-00:20:06 elapsed,  156 min CPU time (~2.1% of a core, continuously)
/home/ole/workspace/pixelwerte/experiments/prism/main/target/debug/prism
```

Unoptimized debug build, burning ~1/6 of a core around the clock since Aug 19.
Looks like a leftover experiment stuck in a loop rather than intentional
background work. Highest-value, lowest-risk fix found: kill it (or find out
why it never exits).

### 2. Three displays, integrated graphics only

Built-in `eDP-1` (1920x1200@60), external LG UltraWide `DP-3` (3440x1440@60),
and an HP X34 on `HDMI-1` — all scale 1, so no fractional-scaling compositing
tax. But that's ~13M pixels composited by GNOME/Mutter on Wayland through a
28W Iris Xe iGPU, at the same time as 3x Claude Code, ~8 Chromium/Electron
renderer processes, 2x tsserver, and a Vite dev server. No dedicated GPU to
offload to. Most plausible explanation for UI stutter / FPS drops during
window drags, workspace switches, video.

### 3. Power profile is "balanced", not "performance"

Cheap and reversible to try while on AC, but expectations should be modest:
the CPU governor is already `powersave` under `intel_pstate` **active** mode
with EPP `balance_performance` — the modern scheme that scales to 5.2GHz on
demand, not the old-style laggy powersave governor. There isn't much obvious
headroom being left on the table here.

### 4. General multitasking load is just heavy

Load average 6.67/5.53/3.20 on 20 logical CPUs (~33% equivalent utilization)
isn't alarming alone, but stacks with #1 and #2: 3 concurrent Claude Code
sessions (~650/587/553MB RSS), ~8 renderer processes from a separate bundled
Chromium (Beeper or similar), 2 TypeScript language servers, an active Vite/
esbuild dev server. All legitimate, all competing for the same iGPU/CPU as
the desktop shell.

### 5. 16-day uptime

Long enough for stray processes (#1) and general fragmentation to build up. A
reboot after cleaning up #1 gives a clean baseline to judge whether the above
actually explains the sluggishness.

## Secondary findings

| Item | Note |
| --- | --- |
| btrfs COW on dev dirs | root/home is one btrfs subvol, already well-tuned (`compress=zstd:1, ssd, discard=async, space_cache=v2`). No `chattr +C` on `~/workspace`'s `node_modules`/`target` build caches — COW + checksumming adds write amplification there. Affects build throughput more than desktop FPS; optional win for a dev machine. |
| journal disk usage | 3.6GB accumulated. Disk-space cleanup only, no perf impact. |
| `fstrim.service` blame | ~2min in `systemd-analyze blame`, unusual given `discard=async` is already live. Likely a one-off/weekly cost from a large extent count (549GB used). Didn't block boot (graphical.target at ~10.4s) — not costing interactive responsiveness today. |
| swappiness = 60 (default) | Non-issue with 45GB available RAM and zram-backed swap. Lowering it is marginal, not a fix for anything observed. |
| thermald/TLP/auto-cpufreq inactive | No throttling seen in `dmesg`/`sensors` (package ~64°C, cores ~56–63°C) under current load. Not causing today's sluggishness, but there's no proactive thermal management for sustained full-load scenarios (gaming, long calls). |

## Confirmed *not* the problem

- NVMe IO scheduler is `none` — correct for NVMe.
- `fstrim.timer` enabled and running weekly; `discard=async` also live.
- Swap is on zram (8GB, compressed) — correct for a 64GB-RAM laptop, not disk swap.
- GPU driver in use is `i915` (correct/stable for this Raptor Lake-P chip); the
  also-loaded `xe` module is harmless/inactive.
- No thermal throttling in `dmesg`; no failed systemd units; boot is healthy
  (~30s total, ~10.4s to graphical target).
- Monitor scaling is integer (1x) on all three displays.
- Docker uses the `overlayfs` storage driver (not the slow btrfs graphdriver);
  0 containers running at analysis time.
- SELinux is Enforcing — negligible desktop cost, not worth trading for speed.

## Non-goals

- No changes were made during this analysis — it's a read-only diagnostic pass.
- Not attempting to fix perceived slowness by disabling security features
  (SELinux) or swapping filesystems — the measured cost of both is low
  relative to items 1–2 above.

## Suggested next step

1. Investigate/kill the two `prism` debug processes — find out why they're
   still running (hung test, forgotten `cargo run`, watch mode).
2. Reboot for a clean baseline.
3. Re-judge desktop feel before touching GNOME animations, power profile, or
   btrfs COW flags — items 2–5 above are plausible contributors but unproven
   until #1's load is out of the picture.
