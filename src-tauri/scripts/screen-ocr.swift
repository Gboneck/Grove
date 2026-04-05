#!/usr/bin/env swift
// Grove OS — Screen capture + OCR via Apple Vision framework
// Captures the main display, runs on-device text recognition, outputs JSON.
// No network calls. No image storage. Text only.

import Cocoa
import Vision

struct ScreenContext: Codable {
    let app: String
    let title: String
    let text: String
    let timestamp: String
}

// Get frontmost app info
func frontmostApp() -> (String, String) {
    let ws = NSWorkspace.shared
    guard let app = ws.frontmostApplication else { return ("unknown", "") }
    let name = app.localizedName ?? app.bundleIdentifier ?? "unknown"

    // Get window title via accessibility (best-effort)
    var title = ""
    if let pid = Optional(app.processIdentifier) {
        let appRef = AXUIElementCreateApplication(pid)
        var value: AnyObject?
        if AXUIElementCopyAttributeValue(appRef, kAXFocusedWindowAttribute as CFString, &value) == .success {
            let window = value as! AXUIElement
            var titleValue: AnyObject?
            if AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &titleValue) == .success {
                title = titleValue as? String ?? ""
            }
        }
    }
    return (name, title)
}

// Capture main display
func captureScreen() -> CGImage? {
    let displayID = CGMainDisplayID()
    return CGDisplayCreateImage(displayID)
}

// Run Vision OCR on image
func recognizeText(from image: CGImage) -> String {
    let semaphore = DispatchSemaphore(value: 0)
    var result = ""

    let request = VNRecognizeTextRequest { req, error in
        guard let observations = req.results as? [VNRecognizedTextObservation] else {
            semaphore.signal()
            return
        }
        let lines = observations.compactMap { obs -> String? in
            obs.topCandidates(1).first?.string
        }
        result = lines.joined(separator: "\n")
        semaphore.signal()
    }
    request.recognitionLevel = .fast  // .fast for speed, .accurate for quality
    request.usesLanguageCorrection = false  // Faster without

    let handler = VNImageRequestHandler(cgImage: image, options: [:])
    try? handler.perform([request])
    semaphore.wait()

    return result
}

// Truncate to keep output manageable
func truncate(_ s: String, maxLen: Int = 1500) -> String {
    if s.count <= maxLen { return s }
    return String(s.prefix(maxLen)) + "..."
}

// Main
let (appName, windowTitle) = frontmostApp()

guard let screenshot = captureScreen() else {
    let ctx = ScreenContext(app: appName, title: windowTitle, text: "", timestamp: ISO8601DateFormatter().string(from: Date()))
    let data = try! JSONEncoder().encode(ctx)
    print(String(data: data, encoding: .utf8)!)
    exit(0)
}

let rawText = recognizeText(from: screenshot)

let ctx = ScreenContext(
    app: appName,
    title: windowTitle,
    text: truncate(rawText),
    timestamp: ISO8601DateFormatter().string(from: Date())
)

let encoder = JSONEncoder()
encoder.outputFormatting = .sortedKeys
let data = try! encoder.encode(ctx)
print(String(data: data, encoding: .utf8)!)
