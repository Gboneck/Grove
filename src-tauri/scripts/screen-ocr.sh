#!/bin/bash
# Grove OS — Screen capture + OCR via macOS native tools
# Captures screen, extracts text via Vision framework (through shortcuts/python), outputs JSON.
# No external dependencies. Everything stays local.

set -e

TMPFILE=$(mktemp /tmp/grove-screen-XXXXXX.png)
trap "rm -f $TMPFILE" EXIT

# 1. Get frontmost app name and window title
APP_NAME=$(osascript -e 'tell application "System Events" to get name of first process whose frontmost is true' 2>/dev/null || echo "unknown")
WINDOW_TITLE=$(osascript -e 'tell application "System Events" to get name of front window of first process whose frontmost is true' 2>/dev/null || echo "")

# 2. Capture screen (silent, no sound, no shadow)
screencapture -x -C "$TMPFILE" 2>/dev/null

# 3. OCR via Python + Vision framework (built into macOS)
OCR_TEXT=$(python3 -c "
import objc
from Foundation import NSURL, NSData
from Quartz import CGImageSourceCreateWithURL, CGImageSourceCreateImageAtIndex
import Vision

url = NSURL.fileURLWithPath_('$TMPFILE')
request = Vision.VNRecognizeTextRequest.alloc().init()
request.setRecognitionLevel_(1)  # 0=accurate, 1=fast
request.setUsesLanguageCorrection_(False)

handler = Vision.VNImageRequestHandler.alloc().initWithURL_options_(url, None)
handler.performRequests_error_([request], None)

results = request.results() or []
lines = []
for obs in results:
    candidates = obs.topCandidates_(1)
    if candidates:
        lines.append(str(candidates[0].string()))

text = chr(10).join(lines)
# Truncate to 1500 chars
if len(text) > 1500:
    text = text[:1500] + '...'
print(text)
" 2>/dev/null || echo "")

# 4. Output JSON
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Escape for JSON
APP_NAME_ESC=$(echo "$APP_NAME" | sed 's/\\/\\\\/g; s/"/\\"/g')
WINDOW_TITLE_ESC=$(echo "$WINDOW_TITLE" | sed 's/\\/\\\\/g; s/"/\\"/g')
OCR_TEXT_ESC=$(echo "$OCR_TEXT" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))" 2>/dev/null || echo '""')

echo "{\"app\":\"$APP_NAME_ESC\",\"title\":\"$WINDOW_TITLE_ESC\",\"text\":$OCR_TEXT_ESC,\"timestamp\":\"$TIMESTAMP\"}"
