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

import { joinPath, startsWithProtocol } from "../utils";
import qs from "qs";

export type Params = Record<string, string | number | boolean> | null;
export type Form = FormData | Record<string, any> | string | any[] | null;
export type Pager<T> = {
  total: number;
  data?: T[] | null;
};
export type IdOnly = { id: string };

export class RequestProvider {
  token: string | null = null;
  baseURL: string;

  constructor(baseURL: string) {
    if (!baseURL.endsWith("/")) {
      baseURL += "/";
    }

    this.baseURL = baseURL;
  }

  async get(
    url: string,
    params: Params = null,
    options: {
      withoutAuth?: boolean;
      headers?: Record<string, string>;
    } = {},
  ): Promise<any> {
    return this.fetchWithOptions({
      method: "GET",
      ...options,
      url,
      params,
    });
  }

  async post(
    url: string,
    form: Form = null,
    options: {
      params?: Params;
      withoutAuth?: boolean;
      headers?: Record<string, string>;
    } = {},
  ): Promise<any> {
    return this.fetchWithOptions({
      method: "POST",
      ...options,
      url,
      form,
    });
  }

  async put(
    url: string,
    form: Form = null,
    options: {
      params?: Params;
      withoutAuth?: boolean;
      headers?: Record<string, string>;
    } = {},
  ): Promise<any> {
    return this.fetchWithOptions({
      method: "PUT",
      ...options,
      url,
      form,
    });
  }

  async delete(
    url: string,
    params: Params = null,
    options: {
      withoutAuth?: boolean;
      headers?: Record<string, string>;
    } = {},
  ): Promise<any> {
    return this.fetchWithOptions({
      method: "DELETE",
      ...options,
      url,
      params,
    });
  }

  // --------------
  // Fetch
  // --------------

  async fetch(
    url: string,
    params: Params,
    init: RequestInit,
  ): Promise<Record<string, any> | ReadableStream<Uint8Array> | null> {
    url = this.getUrl(url, params);

    try {
      const res = await fetch(url, init);
      if (res.status < 200 || res.status >= 400) {
        throw res;
      }
      if (res.headers.get("Content-Type")?.includes("application/json")) {
        return res.json();
      } else {
        return res.body;
      }
    } catch (e) {
      throw await this.getErrorMsg(e);
    }
  }

  async fetchWithOptions(options: {
    method: string;
    url: string;
    params?: Params;
    form?: Form;
    withoutAuth?: boolean;
    headers?: Record<string, string>;
  }): Promise<Record<string, any> | ReadableStream<Uint8Array> | null> {
    const {
      method,
      url,
      params = null,
      form = null,
      withoutAuth = false,
      headers = {},
    } = options;

    const init: RequestInit = { method, headers };
    if (
      this.token != null &&
      !withoutAuth &&
      !Reflect.has(headers, "Authorization")
    ) {
      headers["Authorization"] = `Bearer ${this.token}`;
    }

    if (form instanceof FormData) {
      init.body = form;
    } else if (form != null) {
      init.body = JSON.stringify(form);
      if (!Reflect.has(headers, "Content-Type")) {
        headers["Content-Type"] = "application/json";
      }
    }
    return this.fetch(url, params, init);
  }

  getUrl(url: string, params: Params = null) {
    if (!startsWithProtocol(url)) {
      url = joinPath(this.baseURL, url);
    }

    if (params != null && Reflect.ownKeys(params).length !== 0) {
      url += url.includes("?") ? "&" : "?";
      url += qs.stringify(params);
    }

    return url;
  }

  async getErrorMsg(e: any): Promise<string> {
    if (e instanceof Response) {
      // service error
      try {
        const body = await e.json();
        const msg = body?.msg ?? body?.message ?? "error: system down";
        return msg;
      } catch {
        return "error: system down";
      }
    } else if (!navigator.onLine) {
      // offline
      return "error: network down";
    } else {
      // request abort
      return "error: request abort";
    }
  }

  setToken(token: string | null) {
    this.token = token;
  }
}

export class Module {
  api: RequestProvider;

  constructor(provider: RequestProvider) {
    this.api = provider;
  }
}
