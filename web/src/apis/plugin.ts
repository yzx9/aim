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

import { inject, type Plugin } from "vue";
import { RequestProvider } from "./base";

const API_SYMBOL = Symbol("api");

export interface APIs {
  provider: RequestProvider;
  setToken: (token: string | null) => void;
}

export function useAPIs(): APIs {
  const api = inject<APIs>(API_SYMBOL);
  if (!api) {
    throw new Error("APIs not properly injected in app.");
  }
  return api;
}

export const createAPIs = (baseURL: string): Plugin => ({
  install: (app) => {
    const provider = new RequestProvider(baseURL);

    const apiInstance: APIs = {
      provider: provider,

      setToken: (token: string | null) => {
        provider.token = token;
      },
    };

    app.provide(API_SYMBOL, apiInstance);
  },
});
