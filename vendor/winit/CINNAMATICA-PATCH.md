# Pinned window dependency

Source: https://github.com/cinnabar-mc/winit at
`b1e59b0d5a63628cd4946575765a6c35d53d90df` (0.30.13).
The upstream LICENSE is retained. Mobile and other Cinnabar patches are preserved.

Local change: Windows MonitorHandle tolerates GetMonitorInfo failure in name,
native_identifier and size, as position, refresh rate and video modes already do.
An invalidated monitor returns no name and zero extent rather than panicking.
This addresses Win32 error 1461 in report 20261126-190232; it is unrelated to
the separately reported Pion mDNS core crash.
