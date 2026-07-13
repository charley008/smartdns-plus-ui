FROM pymumu/smartdns:latest

# Keep executable plugin code in the image layer. NAS data volumes may be
# mounted with noexec, which prevents dlopen from loading a bind-mounted .so.
COPY dist/smartdns_plus_ui.so /usr/lib/smartdns/smartdns_plus_ui.so
COPY dist/wwwroot/ /usr/share/smartdns-plus/wwwroot/
