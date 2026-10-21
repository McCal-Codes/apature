# Apature Pocket: Portable USB camera ingest receiver

Date: 2026-10-20. Status: **Phase A prototype only**. No Z6 II physical-hardware verification.

## Why this fits Apature now

The Apature repository already includes `apature-ingest::copy_verified` (SHA-256 copy verification, no overwrites/deletes), a robust plan/run ingest workflow, RAW+JPEG pairing and desktop Live Ingest (two-poll stability). Its `docs/PLAN.md` documents the October 20 Live Ingest release. Pocket should be an optional **USB camera source** feeding Apature, not a second desktop ingest implementation.

A standalone, standard-library Python prototype lives at `experiments/apature-pocket/`. It is deliberately NOT a production Linux receiver and is not added to the Cargo workspace. This avoids changing the existing application while we prove the camera protocol.

## Documented upstream behavior vs unverified behavior

Documented: `gphoto2 --capture-tethered` / `--wait-event-and-download` can wait for camera-generated file events and download images on supported devices; its hook receives `ACTION=download` and `ARGUMENT=<local path>` only after the local download. Source: https://github.com/gphoto/gphoto2/blob/master/doc/gphoto2.1

Unverified: the Nikon Z6 II reports physical-shutter FILEADDED events with this configuration; has no shutter lag/regressions; simultaneously retains full NEF+JPEG originals on its cards; handles multi-shot bursts while tethered; and safely recovers missing shots after reconnect. The `--keep` option alone does not establish these facts. **These are release-blocking questions.**

## Hardware proof-of-concept configuration

- Raspberry Pi 4 Model B (2GB or better), microSD with Raspberry Pi OS Lite.
- USB-A host connection to camera USB-C via a short camera-safe data cable and cable relief.
- USB3 SSD on a separate USB host port, formatted ext4 for Phase A (hardlink commit requires filesystem support).
- Quality 5.1V/3A USB-C battery-backed supply. Budget power across Pi + SSD + camera; Raspberry Pi 4 downstream USB ports share a total ~1.2A peripheral budget, and SSD/enclosure peak draw needs measurement.
- Protective case; no soldering in the first prototype. Do not add GPIO or power-management circuitry until the transport is proven.

Source: Raspberry Pi power and USB documentation, https://www.raspberrypi.com/documentation/computers/raspberry-pi.html

## Acceptance checklist

| Acceptance test | Current state |
| --- | --- |
| Camera recognized by libgphoto2 over USB | Unverified on user's Z6 II |
| Physical shutter works with tether session open | Unverified |
| All original RAW/JPEG saved on camera card during receiver ingest | Unverified; critical |
| New image reported by gphoto2 hook | Unverified |
| Spool file copied and independently SHA-256 checked | Python proof of concept tested with dummy files |
| No overwrite on destination collision | Python proof of concept tested |
| Reprocessing same queued event leaves one archived file | Python proof of concept tested |
| Source spool file survives ingest | Python proof of concept tested |
| Interrupted-copy/process/SSD full/power loss | Partial unit tests only; no hardware qualification |
| Battery runtime, transfer lag and burst throughput | Unmeasured |

## Production architecture when feasibility is established

`Nikon/PTP CameraBackend -> CameraEvents -> durable queue -> transfer worker -> apature-ingest::copy_verified -> Pocket manifest -> Apature desktop import`

Use capability negotiation, not model-name assumptions; keep transport specificities behind the backend boundary. Investigate Nikon official SDK and libgphoto2 FFI after CLI behavior is measured. Require an atomic transaction between final file and manifest, persistent retries with bounded backoff, camera card reconciliation where protocol allows, log rotation, disk mount checks, battery management, and safe shutdown.

## Known risks and decisions

- Tethered USB can change camera behavior. Verify on **test pictures**, not assignments.
- Camera original integrity cannot be proven from spool-to-archive hashing alone.
- The prototype has no physical hold-up capacitor/UPS: Raspberry Pi battery loss may corrupt active state. ext4 plus fsync reduces but does not eliminate that risk.
- A Pi 4 is a development platform, not the final camera-mounted enclosure; USB plug strain and field cable reliability matter more than miniaturization.
- Removable SSD-to-Mac import is the initial handoff. Do not assume a Pi 4 power port supports arbitrary USB gadget storage mode.

## Lessons from Folio

Use Folio's architecture standards as principles: state owns the truth, capability-based behavior, recoverable device errors, documented tests at each layer, and do not describe a feature as verified until real hardware proves it. No Android-specific framework belongs in Pocket.