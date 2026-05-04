/* @refresh reload */
import "./index.css";
import "solid-devtools";

import { createRouter, RouterProvider } from "@tanstack/solid-router";
import { render } from "solid-js/web";

import { AppProvider } from "./api";
import { Toaster } from "./components/toaster";
import { routeTree } from "./routeTree.gen";

const root = document.querySelector("#root");

const router = createRouter({ routeTree });

declare module "@tanstack/solid-router" {
  interface Register {
    router: typeof router;
  }
}

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

render(() => (
  <AppProvider>
    <RouterProvider
      router={router}
      defaultErrorComponent={error => (
        <div>
          <div>
            Error:
            {error.error.message}
          </div>
          <div>
            Stack:
            {error.error.stack}
          </div>
        </div>
      )}
    />
    {/* <Toaster /> */}
  </AppProvider>
), root!);
