/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

/* eslint-disable @typescript-eslint/no-explicit-any */
import type { AxiosInstance, AxiosRequestConfig } from "axios";
import axios from "axios";

/**
 * Rutas desde las cuales NO debemos disparar un redirect "a login con next_path":
 *
 * - `/`                 → es el sign-in / landing de NON_AUTHENTICATED. Redirigir aquí
 *                         genera el loop clásico: login → `/?next_path=X` → replace a
 *                         `/?next_path=/?next_path=X` anidado y/o re-reload infinito.
 * - `/sign-up`, `/accounts/*`, `/auth/*` → pantallas de auth. Un 401 aquí es esperable
 *   durante el flow (ej. validar credenciales), no motivo para reload.
 *
 * Rutas autenticadas (`/onboarding`, `/invitations`, `/workspace-invitations`,
 * `/[workspaceSlug]/*`, etc.) NO están aquí a propósito: un 401 real en ellas sí
 * debe mandar a login con `next_path`, que es el contrato original del interceptor.
 *
 * Regex anclados al inicio, sin trailing slash obligatorio.
 */
const AUTH_LANDING_ROUTES = [/^\/$/, /^\/sign-up(\/|$)/, /^\/accounts(\/|$)/, /^\/auth(\/|$)/];

const isAuthLandingRoute = (pathname: string): boolean => AUTH_LANDING_ROUTES.some((re) => re.test(pathname));

/**
 * Guard a nivel módulo: `WorkspaceAuthWrapper` dispara ~8 `useSWR` en paralelo. Si dos o más
 * caen con 401 a la vez, cada `onRejected` ejecutaría `window.location.replace` — el navegador
 * sólo honra el primero pero los siguientes siguen ejecutándose en microtasks y pueden
 * sobrescribir el destino o re-entrar al handler. Con este flag la primera 401 marca
 * "ya se disparó navegación dura" y las siguientes caen por el early-return.
 *
 * Se resetea solo: en cuanto el browser honre el `replace`, el módulo se re-evalúa en la
 * nueva carga y el flag vuelve a `false`.
 */
let hasDispatched401Redirect = false;

export abstract class APIService {
  protected baseURL: string;
  private axiosInstance: AxiosInstance;

  constructor(baseURL: string) {
    this.baseURL = baseURL;
    this.axiosInstance = axios.create({
      baseURL,
      withCredentials: true,
    });

    this.setupInterceptors();
  }

  private setupInterceptors() {
    this.axiosInstance.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error?.response?.status === 401 && typeof window !== "undefined") {
          const { pathname, search, hash } = window.location;

          // 1. Ya disparamos un redirect en este ciclo de vida del módulo — no duplicar.
          // 2. Estamos en una landing de auth (incluyendo `/`) — redirigir a `/?next_path=/`
          //    generaría loop o un `next_path` inútil. El wrapper de auth ya maneja el estado
          //    ahí; propagar el error y salir.
          // 3. Ya hay un `next_path` puesto apuntando a una ruta autenticada — otra request
          //    paralela ya disparó el flow, no pisar.
          const alreadyHasNextPath = new URLSearchParams(search).has("next_path");

          if (hasDispatched401Redirect || isAuthLandingRoute(pathname) || alreadyHasNextPath) {
            return Promise.reject(error);
          }

          hasDispatched401Redirect = true;
          // Preservamos pathname, search y hash para que el retorno post-login mantenga
          // contexto de filtros, tabs y anchors. `encodeURIComponent` evita romper el
          // parseo del query string del landing.
          const currentPath = `${pathname}${search}${hash}`;
          window.location.replace(`/?next_path=${encodeURIComponent(currentPath)}`);
        }
        return Promise.reject(error);
      }
    );
  }

  get(url: string, params = {}, config: AxiosRequestConfig = {}) {
    return this.axiosInstance.get(url, {
      ...params,
      ...config,
    });
  }

  post(url: string, data = {}, config: AxiosRequestConfig = {}) {
    return this.axiosInstance.post(url, data, config);
  }

  put(url: string, data = {}, config: AxiosRequestConfig = {}) {
    return this.axiosInstance.put(url, data, config);
  }

  patch(url: string, data = {}, config: AxiosRequestConfig = {}) {
    return this.axiosInstance.patch(url, data, config);
  }

  delete(url: string, data?: any, config: AxiosRequestConfig = {}) {
    return this.axiosInstance.delete(url, { data, ...config });
  }

  request(config = {}) {
    return this.axiosInstance(config);
  }
}
