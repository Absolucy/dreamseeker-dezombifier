# DreamSeeker Dezombifier

BYOND's "DreamSeeker" client has a notable issue right now where, whenever reconnecting/relaunching to a server, sometimes it'll leave invisible "zombie" processes running in the background. These can take up memory, and over time, stack up to the point where it can slow even my PC (Ryzen 9 5900X, 64GB RAM) to a crawl.

This is a simple service that finds and terminates those zombie processes. It kills processes that meet the following criteria:
- Process named `dreamseeker.exe`
- Started more than 5 minutes ago
  - This is to avoid issues where it might kill a DS process during the short period between the loading screen and opening the actual game window.
- No visible windows associated with process
  - Note that minimized windows are still considered visible.

It requires no configuration, and just does its thing in the background. It only uses 1000~ KB of memory while running, and barely any CPU usage.
