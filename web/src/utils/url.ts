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
