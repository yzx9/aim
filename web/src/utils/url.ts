// Copyright 2025 Zexin Yuan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

export function joinPath(base: string, path: string): string {
  if (!base) {
    return path;
  }
  if (!path) {
    return base;
  }
  if (base.endsWith("/") && path.startsWith("/")) {
    return base + path.slice(1);
  } else if (!base.endsWith("/") && !path.startsWith("/")) {
    return base + "/" + path;
  } else {
    return base + path;
  }
}

const EXTERNAL_URL_RE = /^https?:/i;

export function startsWithProtocol(url: string): boolean {
  return EXTERNAL_URL_RE.test(url);
}

if (import.meta.vitest) {
  const { describe, it, expect } = import.meta.vitest;

  describe("joinPath", () => {
    it("should join base and path with a slash when neither has one", () => {
      expect(joinPath("base", "path")).toBe("base/path");
    });

    it("should join base and path without adding a slash when base has a trailing slash", () => {
      expect(joinPath("base/", "path")).toBe("base/path");
    });

    it("should join base and path without adding a slash when path has a leading slash", () => {
      expect(joinPath("base", "/path")).toBe("base/path");
    });

    it("should join base and path without adding a slash when both have slashes", () => {
      expect(joinPath("base/", "/path")).toBe("base/path");
    });

    it("should handle empty base", () => {
      expect(joinPath("", "path")).toBe("path");
    });

    it("should handle empty path", () => {
      expect(joinPath("base", "")).toBe("base");
    });

    it("should handle empty base and path", () => {
      expect(joinPath("", "")).toBe("");
    });
  });

  describe("startsWithProtocol", () => {
    it("should return true for http:// URLs", () => {
      expect(startsWithProtocol("http://example.com")).toBe(true);
    });

    it("should return true for https:// URLs", () => {
      expect(startsWithProtocol("https://example.com")).toBe(true);
    });

    it("should return false for relative URLs", () => {
      expect(startsWithProtocol("/path/to/resource")).toBe(false);
    });

    it("should return false for URLs without a protocol", () => {
      expect(startsWithProtocol("example.com")).toBe(false);
    });

    it("should return false for empty strings", () => {
      expect(startsWithProtocol("")).toBe(false);
    });
  });
}
