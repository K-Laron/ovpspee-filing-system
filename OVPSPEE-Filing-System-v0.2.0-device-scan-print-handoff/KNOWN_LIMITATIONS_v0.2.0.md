# Known Limitations v0.2.0

- QR label print deferred.
- Report print deferred.
- OCR not included.
- TWAIN not included.
- Reliable ADF/multi-page scan deferred.
- PDF scan output batching deferred.
- Advanced print options deferred:
  - page range
  - duplex
  - color mode
  - paper size
- Background printer/scanner polling deferred.
- Cloud/web server deployment not included.
- Device UI may show scanner port string such as `\\.\Usbscan0`.
- Restore destructive flow was not manually re-executed during feature audit; automated restore tests passed and controls were visible.
- Backup UI still shows local backup folder paths to Admin; existing behavior, not changed in this release.

